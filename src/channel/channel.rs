//---------------------------------------------------------------------------------------------------- Sender
pub trait SansanSender<T> {
	type Error;
	fn try_send(&self, t: T) -> Result<(), Self::Error>;
}

//---------------------------------------------------------------------------------------------------- Receiver
pub trait SansanReceiver<T> {
	type Error;
	fn try_recv(&self) -> Result<T, Self::Error>;
}

//---------------------------------------------------------------------------------------------------- Crossbeam
impl<T> SansanSender<T> for crossbeam::channel::Sender<T> {
	type Error = crossbeam::channel::TrySendError<T>;
	#[inline(always)]
	fn try_send(&self, t: T) -> Result<(), Self::Error> {
		self.try_send(t)
	}
}
impl<T> SansanReceiver<T> for crossbeam::channel::Receiver<T> {
	type Error = crossbeam::channel::TryRecvError;
	#[inline(always)]
	fn try_recv(&self) -> Result<T, Self::Error> {
		self.try_recv()
	}
}

//---------------------------------------------------------------------------------------------------- STD
impl<T> SansanSender<T> for std::sync::mpsc::Sender<T> {
	type Error = std::sync::mpsc::SendError<T>;
	#[inline(always)]
	fn try_send(&self, t: T) -> Result<(), Self::Error> {
		self.send(t)
	}
}
impl<T> SansanSender<T> for std::sync::mpsc::SyncSender<T> {
	type Error = std::sync::mpsc::TrySendError<T>;
	#[inline(always)]
	fn try_send(&self, t: T) -> Result<(), Self::Error> {
		self.try_send(t)
	}
}
impl<T> SansanReceiver<T> for std::sync::mpsc::Receiver<T> {
	type Error = std::sync::mpsc::TryRecvError;
	#[inline(always)]
	fn try_recv(&self) -> Result<T, Self::Error> {
		self.try_recv()
	}
}

// pub type SansanSender<T>   = crossbeam::channel::Sender<T>;
// pub type SansanReceiver<T> = crossbeam::channel::Receiver<T>;

// pub type SansanSender<T>   = std::sync::mpsc::Sender<T>;
// pub type SansanReceiver<T> = std::sync::mpsc::Receiver<T>;