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






