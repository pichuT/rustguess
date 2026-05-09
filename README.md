# rustguess — a number-guessing game as a Linux kernel module

A Linux kernel character device that runs a number-guessing game at `/dev/rustguess`, written in safe Rust.

Licensed GPL-2.0 to match the Linux kernel.

## Demo
```
$ sudo cat /dev/rustguess
Welcome! Guess a number between 1 and 100. `echo N > /dev/rustguess`, then `cat /dev/rustguess`.

$ echo 50 | sudo tee /dev/rustguess > /dev/null
$ sudo cat /dev/rustguess
50 is too high -- guess lower.

$ echo 25 | sudo tee /dev/rustguess > /dev/null
$ sudo cat /dev/rustguess
25 is too low -- guess higher.

$ echo 42 | sudo tee /dev/rustguess > /dev/null
$ sudo cat /dev/rustguess
Correct! You got it in 3 tries.

$ echo 50 | sudo tee /dev/rustguess > /dev/null
$ sudo cat /dev/rustguess
You already won! `rmmod rustguess && insmod rustguess.ko` to play again.
```

## What this is
`rustguess` is a Linux kernel module that runs a number-guessing game. You write guesses in, you read hints back. The game state persists until you unload and reload the module. This project is a small example of how kernel code can host user-facing protocols at Ring 0, written entirely in Rust.

## Build & Run
**Set up the VM and toolchain:**

```bash
multipass launch --name hw5 lts
multipass shell hw5

sudo apt update
sudo apt install -y build-essential linux-headers-$(uname -r) kmod
sudo apt install -y rustc-1.93 rust-1.93-src bindgen
sudo update-alternatives --install /usr/bin/rustc rustc /usr/bin/rustc-1.93 100
```

**Build and load:**

```bash
git clone https://github.com/pichuT/rustguess
cd rustguess
make
sudo insmod rustguess.ko
```

**Play:**

```bash
sudo cat /dev/rustguess                        # welcome message
echo 50 | sudo tee /dev/rustguess > /dev/null  # make a guess
sudo cat /dev/rustguess                        # read the hint
```

**Unload:**

```bash
sudo rmmod rustguess
```

## Code Tour
`impl kernel::InPlaceModule for RustGuess` — that is where the global game state is initialized and the device is registered via `MiscDeviceRegistration`.
`write_iter` — that is where the interesting work happens. It drains the user's input bytes, parses them as a `u64`, locks the global game state, and picks a response based on whether the guess is too low, too high, or correct. 
`match` covers every case — Rust's type system requires every `Option` variant to be handled, so there is no path where a malformed guess goes unhandled.
`read_iter` delivers the response back to user space. It uses a per-open `served` alongside the global `consumed` flag to make `cat` exit cleanly after reading the current message once, without looping forever.

## Design Notes
**Why `global_lock!` + `Mutex<GameState>`?** The game state is shared across all opens of `/dev/rustguess` — every `echo` and every `cat` sees the same in-progress game. The `global_lock!` enforces that you cannot touch `GAME` without holding the lock; the guard auto-releases on scope exit. You cannot accidentally access the game state unprotected as the compiler prevents it.

**Why a per-open `served` flag?** Without it, `cat` would call `read()` in a loop and see the same message forever. The `served` flag marks when this particular open has already streamed the current message, so the next `read()` returns 0 (EOF) and `cat` exits cleanly.

**Why `KVec<u8>` instead of a fixed array?** Messages vary in length depending on the guess value and try count. `KVec` is buffer where fallible allocation via `extend_from_slice(..., GFP_KERNEL)?` means allocation failures propagate as errors rather than panicking the kernel.

**Why no `Drop` implementation?** Cleanup is handled automatically. The `MiscDeviceRegistration` field has its own `Drop` that deregisters the device when the module is unloaded. There is no cleanup function to forget.

## Future Work
- **Random secret.** Use `kernel::random::getrandom` to pick a fresh secret at module load instead of hardcoding `42`.
- **Difficulty levels.** Write `RANGE:1000\n` to expand the search space before guessing.
- **Cheat code.** A `REVEAL\n` command in debug mode that prints the secret.





