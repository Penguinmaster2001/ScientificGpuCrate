use crate::app::{
    input::InputEvent,
    scig_env::{
        FrameData,
        ScigEnv,
    },
    state::FrameState,
};
use pollster::FutureExt;
use std::{
    sync::mpsc,
    thread::{
        self,
    },
    time::{
        Duration,
        Instant,
    },
};
use tokio::sync::watch::{
    self,
    Ref,
};



type RunFn<E> =
    Box<dyn FnOnce(E, mpsc::Receiver<EnvMessage>, watch::Sender<FrameData>) + Send + 'static>;



pub enum EnvMessage
{
    Start,
    Stop,
    Frame(FrameState),
    Input(InputEvent),
}



pub enum RunMode
{
    None,
    Free,
    Clock
    {
        dt: Duration,
        send_messages_between_updates: bool,
    },
    FrameLock
    {
        step_pre_frame: bool,
    },
}



pub struct EnvRunner
{
    handle: EnvHandle<EnvMessage, FrameData>,
}



impl EnvRunner
{
    pub fn new<E: ScigEnv>(env: E, mode: RunMode) -> Self
    {
        let run: RunFn<E> = match mode
        {
            RunMode::None => Box::new(Self::none_run),
            RunMode::Free => Box::new(Self::free_run),
            RunMode::FrameLock { step_pre_frame } => Self::frame_step_run(step_pre_frame),
            RunMode::Clock {
                dt,
                send_messages_between_updates,
            } => Self::clock(dt, send_messages_between_updates),
        };

        let handle = EnvHandle::new(env, FrameData::default(), run);

        Self { handle }
    }



    pub fn start(&self) -> anyhow::Result<()>
    {
        Ok(self.handle.sender.send(EnvMessage::Start)?)
    }



    pub fn get_frame_immediate(&mut self) -> anyhow::Result<Ref<'_, FrameData>>
    {
        if self.handle.finished()
        {
            anyhow::bail!("thread done")
        }

        Ok(self.handle.snapshot.borrow_and_update())
    }



    pub fn get_frame(&mut self, frame: FrameState) -> anyhow::Result<Ref<'_, FrameData>>
    {
        if self.handle.finished()
        {
            anyhow::bail!("thread done")
        }

        self.handle.sender.send(EnvMessage::Frame(frame))?;

        // TODO: handle missed frame behavior and time limit
        let deadline = std::time::Instant::now() + Duration::from_millis(1);
        while !self.handle.snapshot.has_changed().unwrap_or(false)
            && std::time::Instant::now() < deadline
        {
            std::thread::yield_now();
        }

        Ok(self.handle.snapshot.borrow_and_update())
    }



    fn none_run<E: ScigEnv>(
        _env: E,
        receiver: mpsc::Receiver<EnvMessage>,
        snapshot: watch::Sender<FrameData>,
    )
    {
        loop
        {
            match receiver.recv().unwrap()
            {
                EnvMessage::Frame(_) => snapshot.send_modify(|_| {}),
                EnvMessage::Stop => break,
                _ => (),
            }
        }
    }



    fn frame_step_run<E: ScigEnv>(step_pre_frame: bool) -> RunFn<E>
    {
        Box::new(move |mut env, receiver, snapshot| {
            while let Ok(m) = receiver.recv()
            {
                match m
                {
                    EnvMessage::Frame(frame) =>
                    {
                        if step_pre_frame
                        {
                            env.frame(frame);
                            snapshot.send_modify(|f| {
                                *f = env.get_frame();
                            });
                        }
                        else
                        {
                            snapshot.send_modify(|f| {
                                *f = env.get_frame();
                            });
                            env.frame(frame);
                        }
                    }
                    EnvMessage::Stop => break,
                    _ => (),
                }
            }

            env.stop();
        })
    }



    fn free_run(
        mut env: impl ScigEnv,
        receiver: mpsc::Receiver<EnvMessage>,
        snapshot: watch::Sender<FrameData>,
    )
    {
        let mut prev = Instant::now();

        loop
        {
            let now = Instant::now();
            let delta = now - prev;
            env.step(delta);

            match receiver.try_recv()
            {
                Ok(m) => match m
                {
                    EnvMessage::Start => env.start(),
                    EnvMessage::Stop => break,
                    EnvMessage::Frame(frame) =>
                    {
                        env.frame(frame);
                        snapshot.send_modify(|f| *f = env.get_frame())
                    }
                    EnvMessage::Input(input) => env.input(input),
                },
                Err(e) => match e
                {
                    mpsc::TryRecvError::Empty => (),
                    mpsc::TryRecvError::Disconnected => break,
                },
            };


            prev = now;
        }

        env.stop();
    }



    fn clock<E: ScigEnv>(dt: Duration, _send_messages_between_updates: bool) -> RunFn<E>
    {
        Box::new(move |mut env, receiver, snapshot| {
            let mut interval = tokio::time::interval(dt);

            loop
            {
                interval.tick().block_on();

                match receiver.recv().unwrap()
                {
                    EnvMessage::Start => env.start(),
                    EnvMessage::Stop => break,
                    EnvMessage::Frame(frame) =>
                    {
                        env.frame(frame);
                        snapshot.send_modify(|f| *f = env.get_frame())
                    }
                    EnvMessage::Input(input) => env.input(input),
                }
            }
        })
    }
}



pub struct EnvHandle<M, S>
{
    sender: mpsc::Sender<M>,
    snapshot: watch::Receiver<S>,
    handle: thread::JoinHandle<()>,
}



impl<M: 'static + Send, S: 'static + Send + Sync> EnvHandle<M, S>
{
    pub fn new<F, T: 'static + Send>(state: T, init: S, run: F) -> Self
    where
        F: 'static + Send + FnOnce(T, mpsc::Receiver<M>, watch::Sender<S>),
    {
        let (sender, receiver) = mpsc::channel();
        let (snapshot_sender, snapshot_receiver) = watch::channel(init);

        let handle = thread::spawn(move || run(state, receiver, snapshot_sender));

        Self {
            sender,
            snapshot: snapshot_receiver,
            handle,
        }
    }



    pub fn finished(&self) -> bool
    {
        self.handle.is_finished()
    }
}
