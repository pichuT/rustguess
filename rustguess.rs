// SPDX-License-Identifier: GPL-2.0
//! rustguess: a number-guessing game character device.

use kernel::{
    fs::{File, Kiocb},
    iov::{IovIterDest, IovIterSource},
    miscdevice::{MiscDevice, MiscDeviceOptions, MiscDeviceRegistration},
    new_mutex,
    prelude::*,
    str::CString,
    sync::Mutex,
};

module! {
    type: RustGuess,
    name: "rustguess",
    description: "rustguess — a number-guessing game character device",
    license: "GPL",
}

const SECRET: u64 = 42;
const MAX_GUESS: u64 = 100;

struct GameState {
    last_message: KVec<u8>,
    consumed: bool,
    tries: u64,
    won: bool,
}

kernel::sync::global_lock! {
    unsafe(uninit) static GAME: Mutex<GameState> = GameState {
        last_message: KVec::new(),
        consumed: true,
        tries: 0,
        won: false,
    };
}

fn build_message(bytes: &[u8]) -> Result<KVec<u8>> {
    let mut v: KVec<u8> = KVec::new();
    v.extend_from_slice(bytes, GFP_KERNEL)?;
    Ok(v)
}

#[pin_data]
struct RustGuess {
    #[pin]
    _miscdev: MiscDeviceRegistration<RustGuessDevice>,
}

impl kernel::InPlaceModule for RustGuess {
    fn init(_module: &'static ThisModule) -> impl PinInit<Self, Error> {
        pr_info!("module loaded (secret picked, get guessing!)\n");
        // SAFETY: Called exactly once during module init.
        unsafe { GAME.init() };
        if let Ok(welcome) = build_message(
            b"Welcome! Guess a number between 1 and 100. \
              `echo N > /dev/rustguess`, then `cat /dev/rustguess`.\n",
        ) {
            let mut g = GAME.lock();
            g.last_message = welcome;
            g.consumed = false;
        }
        let opts = MiscDeviceOptions { name: c"rustguess" };
        try_pin_init!(Self {
            _miscdev <- MiscDeviceRegistration::register(opts),
        })
    }
}

#[pin_data]
struct RustGuessDevice {
    #[pin]
    served: Mutex<bool>,
}

#[vtable]
impl MiscDevice for RustGuessDevice {
    type Ptr = Pin<KBox<Self>>;

    fn open(_file: &File, _misc: &MiscDeviceRegistration<Self>) -> Result<Pin<KBox<Self>>> {
        KBox::try_pin_init(
            try_pin_init! { RustGuessDevice { served <- new_mutex!(false) } },
            GFP_KERNEL,
        )
    }

    fn write_iter(mut kiocb: Kiocb<'_, Self::Ptr>, iov: &mut IovIterSource<'_>) -> Result<usize> {
        let mut buf: KVec<u8> = KVec::new();
        let len = iov.copy_from_iter_vec(&mut buf, GFP_KERNEL)?;

        let s = core::str::from_utf8(&buf).unwrap_or("");
        let guess: Option<u64> = s.trim().parse().ok();

        let mut state = GAME.lock();

        if state.won {
            state.last_message = build_message(
                b"You already won! `rmmod rustguess && insmod rustguess.ko` to play again.\n",
            )?;
            state.consumed = false;
            *kiocb.ki_pos_mut() = 0;
            return Ok(len);
        }

        let response_bytes = match guess {
            None => build_message(b"Couldn't parse your input as a number. Try again.\n")?,
            Some(g) if g == 0 || g > MAX_GUESS => {
                build_message(b"Out of range -- pick a number between 1 and 100.\n")?
            }
            Some(g) => {
                state.tries += 1;
                let formatted = if g < SECRET {
                    CString::try_from_fmt(fmt!("{g} is too low -- guess higher.\n"))?
                } else if g > SECRET {
                    CString::try_from_fmt(fmt!("{g} is too high -- guess lower.\n"))?
                } else {
                    state.won = true;
                    CString::try_from_fmt(fmt!("Correct! You got it in {} tries.\n", state.tries))?
                };
                build_message(formatted.to_bytes())?
            }
        };

        state.last_message = response_bytes;
        state.consumed = false;
        *kiocb.ki_pos_mut() = 0;

        pr_info!("tries={}, won={}\n", state.tries, state.won);
        Ok(len)
    }

    fn read_iter(mut kiocb: Kiocb<'_, Self::Ptr>, iov: &mut IovIterDest<'_>) -> Result<usize> {
        let me = kiocb.file();
        let mut served = me.served.lock();

        let state = GAME.lock();
        if state.consumed && *served {
            return Ok(0);
        }
        let n = iov.simple_read_from_buffer(kiocb.ki_pos_mut(), &state.last_message)?;
        if n == 0 {
            *served = true;
        }
        Ok(n)
    }
}
