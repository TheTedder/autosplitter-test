#![no_std]

use core::{hint, mem, panic::PanicInfo};

use arrayvec::ArrayVec;
use asl::{Address, Process, TimerState};
use bytemuck::{Pod, Zeroable};
use spinning_top::{const_spinlock, Spinlock};

#[cfg(target_arch = "wasm32")]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // TODO: Print the panic.
    unsafe { core::arch::wasm32::unreachable() }
}

#[derive(Default)]
struct Watcher<T: Pod> {
    current: T,
    old: T,
}

impl<T: Pod> Watcher<T> {
    fn try_update(&mut self, process: &Process, address: Address) -> Result<(), ()> {
        self.update(process.read(address)?);
        Ok(())
    }

    fn update(&mut self, value: T) {
        self.old = mem::replace(&mut self.current, value);
    }
}

#[derive(Default)]
struct State {
    process: Option<Process>,
    after_logo: Watcher<u8>,
}

static STATE: Spinlock<State> = const_spinlock(State {
    process: None,
    after_logo: Watcher { old: 0, current: 0 },
});

// state("BioshockInfinite")
// {
// 	int overlaysPtr :       0x1415A30, 0x124;
// 	int overlaysCount :     0x1415A30, 0x128;
// }

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Overlays {
    ptr: u32,
    count: i32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Default)]
struct Overlay {
    name_ptr: u32,
    name_len: u32,
}

const BASE_MODULE_OFFSET: u64 = 0x00400000;

const fn abs_addr(a: u64) -> Address {
    Address(a + BASE_MODULE_OFFSET)
}

const fn wide_str<const N: usize>(s: &[u8; N]) -> [u16; N] {
    let mut arr = [0; N];
    let mut i = 0;
    while i < N {
        arr[i] = s[i] as u16;
        i += 1;
    }
    arr
}

static OVERLAY_NAME: [u16; 0x36] =
    wide_str(b"GFXScriptReferenced.GameThreadLoadingScreen_Data_Oct22");

impl State {
    fn update(&mut self) -> Result<(), ()> {
        if self.process.is_none() {
            self.process = Process::attach("BioShockInfinite.exe");
        }
        if let Some(process) = &self.process {
            match asl::timer_state() {
                TimerState::NotRunning => {
                    self.after_logo.try_update(process, abs_addr(0x135697C))?;
                    let any_key = process.read::<u8>(abs_addr(0x13D2AA2))?;
                    if let (1..=255, 1, 0) = (any_key, self.after_logo.current, self.after_logo.old)
                    {
                        asl::start();
                    }
                }
                TimerState::Running => {
                    // This is the variable used to track when map data is being
                    // loaded. This includes load screens and OOB load zones.
                    // Note, this doesn't include the load screen transition
                    // time. We have to look for the overlay otherwise the timer
                    // will be delayed when starting/stoppping.

                    // TODO: We need some sort of .read_ptr() -> Address that
                    // reads either 4 or 8 bytes.
                    let is_map_loading = process.read::<u32>(Address(
                        process.read::<u32>(abs_addr(0x14154E8))? as u64 + 0x4,
                    ))?;

                    if is_map_loading != 0xbf800000 {
                        //asl::pause_game_time();
                        asl::pause_game_time();
                        return Ok(());
                    }

                    let overlays = process.read::<Overlays>(Address(
                        process.read::<u32>(abs_addr(0x1415A30))? as u64 + 0x124,
                    ))?;

                    if overlays.count < 0 || overlays.count > 8 {
                        asl::resume_game_time();
                        return Ok(());
                    }

                    let mut overlay_ptrs = ArrayVec::<u32, 8>::new();
                    unsafe {
                        overlay_ptrs.set_len(overlays.count as usize);
                        process.read_into_slice(Address(overlays.ptr as u64), &mut overlay_ptrs)?;
                    }

                    for overlay_ptr in overlay_ptrs {
                        let overlay = process.read::<Overlay>(Address(overlay_ptr as u64))?;

                        if overlay.name_len - 1 != 0x36 {
                            continue;
                        }

                        let name = process.read::<[u16; 0x36]>(Address(overlay.name_ptr as u64))?;

                        if name == OVERLAY_NAME {
                            asl::pause_game_time();
                            return Ok(());
                        }
                    }

                    asl::resume_game_time();
                }
                TimerState::Finished => {}
            }
        }
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn update() {
    let _ = STATE.lock().update();
}

#[no_mangle]
pub extern "C" fn configure() {
    //asl::start();
    asl::set_tick_rate(500.0);
}
