use pipewire::{
    context::Context,
    keys,
    main_loop::MainLoop,
    properties::properties,
    spa::{
        self,
        param::{
            ParamType,
            format::{MediaSubtype, MediaType},
            format_utils,
        },
        pod::{Object, Pod, Value, serialize::PodSerializer},
        utils::{Direction, SpaTypes},
    },
    stream::{Stream, StreamFlags},
};
use std::{
    mem,
    sync::mpsc,
    thread::{self, JoinHandle},
};

pub enum PipewireOutgoingMessage {
    Terminate,
}

pub enum PipewireIncomingMessage {
    Ready,
    Error(Box<dyn std::error::Error + Send>),
    LeftChannelData(Vec<f32>),
    RightChannelData(Vec<f32>),
}

struct AudioStreamData {
    format: spa::param::audio::AudioInfoRaw,
    data_sender: mpsc::Sender<PipewireIncomingMessage>,
}

pub struct PipewireInput {
    pub from_pipewire: mpsc::Receiver<PipewireIncomingMessage>,
    to_pipewire: pipewire::channel::Sender<PipewireOutgoingMessage>,
    pipewire_thread: Option<JoinHandle<()>>,
}

impl PipewireInput {
    pub fn new() -> Result<PipewireInput, Box<dyn std::error::Error>> {
        let (from_pipewire_tx, from_pipewire_rx) =
            std::sync::mpsc::channel::<PipewireIncomingMessage>();
        let (to_pipewire_tx, to_pipewire_rx) = pipewire::channel::channel();

        let pipewire_thread = thread::spawn(move || {
            let mainloop = match MainLoop::new(None) {
                Ok(mainloop) => mainloop,
                Err(err) => {
                    let _ = from_pipewire_tx.send(PipewireIncomingMessage::Error(Box::new(err)));
                    return;
                }
            };
            let context = match Context::new(&mainloop) {
                Ok(context) => context,
                Err(err) => {
                    let _ = from_pipewire_tx.send(PipewireIncomingMessage::Error(Box::new(err)));
                    return;
                }
            };
            let core = match context.connect(None) {
                Ok(core) => core,
                Err(err) => {
                    let _ = from_pipewire_tx.send(PipewireIncomingMessage::Error(Box::new(err)));
                    return;
                }
            };

            let audio_stream = match Stream::new(
                &core,
                "audio-input",
                properties! {
                    *keys::MEDIA_TYPE => "Audio",
                    *keys::MEDIA_CATEGORY => "Capture",
                    *keys::MEDIA_ROLE => "DSP",
                    *keys::AUDIO_CHANNELS => "2",
                },
            ) {
                Ok(core) => core,
                Err(err) => {
                    let _ = from_pipewire_tx.send(PipewireIncomingMessage::Error(Box::new(err)));
                    return;
                }
            };

            let audio_stream_data = AudioStreamData {
                format: Default::default(),
                data_sender: from_pipewire_tx.clone(),
            };

            let _audio_listener = match audio_stream
                .add_local_listener_with_user_data(audio_stream_data)
                .param_changed(|_, audio_stream_data, id, param| {
                    let Some(param) = param else {
                        return;
                    };
                    if id != ParamType::Format.as_raw() {
                        return;
                    }

                    let (media_type, media_subtype) = match format_utils::parse_format(param) {
                        Ok(v) => v,
                        Err(_) => return,
                    };

                    if media_type != MediaType::Audio || media_subtype != MediaSubtype::Raw {
                        return;
                    }

                    audio_stream_data
                        .format
                        .parse(param)
                        .expect("Failed to parse param changed to AudioInfoRaw");
                })
                .process(|stream, audio_stream_data| match stream.dequeue_buffer() {
                    None => {}
                    Some(mut buffer) => {
                        let datas = buffer.datas_mut();
                        if datas.is_empty() {
                            return;
                        }

                        let data = &mut datas[0];
                        let number_channels = audio_stream_data.format.channels();
                        let number_samples = data.chunk().size() / (mem::size_of::<f32>() as u32);

                        if let Some(samples) = data.data() {
                            let mut left_buffer = Vec::<f32>::new();
                            let mut right_buffer = Vec::<f32>::new();

                            for channel in 0..number_channels {
                                for sample_index in
                                    (channel..number_samples).step_by(number_channels as usize)
                                {
                                    let start = sample_index as usize * mem::size_of::<f32>();
                                    let end = start + mem::size_of::<f32>();
                                    let sample = &samples[start..end];
                                    let sample_float =
                                        f32::from_le_bytes(sample.try_into().unwrap());

                                    match channel {
                                        0 => left_buffer.push(sample_float),
                                        1 => right_buffer.push(sample_float),
                                        _ => {}
                                    }
                                }
                            }

                            let _ = audio_stream_data
                                .data_sender
                                .send(PipewireIncomingMessage::LeftChannelData(left_buffer));
                            let _ = audio_stream_data
                                .data_sender
                                .send(PipewireIncomingMessage::RightChannelData(right_buffer));
                        }
                    }
                })
                .register()
            {
                Ok(audio_listener) => audio_listener,
                Err(err) => {
                    let _ = from_pipewire_tx.send(PipewireIncomingMessage::Error(Box::new(err)));
                    return;
                }
            };

            let mut audio_info = spa::param::audio::AudioInfoRaw::new();
            audio_info.set_format(spa::param::audio::AudioFormat::F32LE);
            let audio_parameters: Vec<u8> = (match PodSerializer::serialize(
                std::io::Cursor::new(Vec::new()),
                &Value::Object(Object {
                    type_: SpaTypes::ObjectParamFormat.as_raw(),
                    id: ParamType::EnumFormat.as_raw(),
                    properties: audio_info.into(),
                }),
            ) {
                Ok(audio_parameters) => audio_parameters,
                Err(err) => {
                    let _ = from_pipewire_tx.send(PipewireIncomingMessage::Error(Box::new(err)));
                    return;
                }
            })
            .0
            .into_inner();

            match audio_stream.connect(
                Direction::Input,
                None,
                StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
                &mut [Pod::from_bytes(&audio_parameters).unwrap()],
            ) {
                Ok(core) => core,
                Err(err) => {
                    let _ = from_pipewire_tx.send(PipewireIncomingMessage::Error(Box::new(err)));
                    return;
                }
            };

            let _receiver = to_pipewire_rx.attach(mainloop.loop_(), {
                let mainloop = mainloop.clone();
                move |_| mainloop.quit()
            });

            let _ = from_pipewire_tx.send(PipewireIncomingMessage::Ready);

            mainloop.run();

            let _ = audio_stream.disconnect();
        });

        match from_pipewire_rx.recv() {
            Ok(message) => {
                if let PipewireIncomingMessage::Error(err) = message {
                    return Err(err);
                }
            }
            Err(err) => {
                return Err(Box::new(err));
            }
        };

        Ok(PipewireInput {
            from_pipewire: from_pipewire_rx,
            to_pipewire: to_pipewire_tx,
            pipewire_thread: Some(pipewire_thread),
        })
    }
}

impl Drop for PipewireInput {
    fn drop(&mut self) {
        let _ = self.to_pipewire.send(PipewireOutgoingMessage::Terminate);
        if let Some(pipewire_thread) = self.pipewire_thread.take() {
            pipewire_thread.join().unwrap();
        }
    }
}
