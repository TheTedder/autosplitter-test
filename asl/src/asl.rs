use bytemuck::Pod;
use core::{
    mem::{self, MaybeUninit},
    slice,
};

mod sys {
    use super::{Address, PointerKind, Process, State};

    pub const INVALID_PROCESS_HANDLE: i64 = 0x1_FFFF_FFFF;

    extern "C" {
        pub fn start();
        pub fn split();
        pub fn reset();
        pub fn attach(name_ptr: *const u8, name_len: usize) -> i64;
        pub fn detach(process: i64);
        // pub fn set_process_name(name_ptr: *const u8, name_len: usize);
        // pub fn push_pointer_path(
        //     module_ptr: *const u8,
        //     module_len: usize,
        //     kind: PointerKind,
        // ) -> usize;
        // pub fn push_offset(pointer_path_id: usize, offset: i64);
        // pub fn get_u8(pointer_path_id: usize, current: State) -> u8;
        // pub fn get_u16(pointer_path_id: usize, current: State) -> u16;
        // pub fn get_u32(pointer_path_id: usize, current: State) -> u32;
        // pub fn get_u64(pointer_path_id: usize, current: State) -> u64;
        // pub fn get_i8(pointer_path_id: usize, current: State) -> i8;
        // pub fn get_i16(pointer_path_id: usize, current: State) -> i16;
        // pub fn get_i32(pointer_path_id: usize, current: State) -> i32;
        // pub fn get_i64(pointer_path_id: usize, current: State) -> i64;
        // pub fn get_f32(pointer_path_id: usize, current: State) -> f32;
        // pub fn get_f64(pointer_path_id: usize, current: State) -> f64;
        // pub fn scan_signature(sig_ptr: *const u8, sig_len: usize) -> Address;
        pub fn set_tick_rate(ticks_per_second: f64);
        pub fn print_message(text_ptr: *const u8, text_len: usize);
        pub fn read_into_buf(
            process: i64,
            address: Address,
            buf_ptr: *mut u8,
            buf_len: usize,
        ) -> bool;
        pub fn set_variable(
            key_ptr: *const u8,
            key_len: usize,
            value_ptr: *const u8,
            value_len: usize,
        );
        pub fn get_timer_state() -> i32;
        pub fn set_game_time(secs: u64, nanos: u32);
        pub fn pause_game_time();
        pub fn resume_game_time();
        pub fn is_game_time_paused() -> i32;
    }
}

#[derive(Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Process(i64);

impl Drop for Process {
    fn drop(&mut self) {
        unsafe { sys::detach(self.0) }
    }
}

impl Process {
    pub fn attach(name: &str) -> Option<Self> {
        let id = unsafe { sys::attach(name.as_ptr(), name.len()) };
        if id != sys::INVALID_PROCESS_HANDLE {
            Some(Self(id))
        } else {
            None
        }
    }

    pub fn read_into_buf(&self, address: Address, buf: &mut [u8]) -> Result<(), ()> {
        unsafe {
            if sys::read_into_buf(self.0, address, buf.as_mut_ptr(), buf.len()) {
                Ok(())
            } else {
                Err(())
            }
        }
    }

    pub fn read<T: Pod>(&self, address: Address) -> Result<T, ()> {
        unsafe {
            let mut value = MaybeUninit::<T>::uninit();
            self.read_into_buf(
                address,
                slice::from_raw_parts_mut(value.as_mut_ptr().cast(), mem::size_of::<T>()),
            )?;
            Ok(value.assume_init())
        }
    }

    pub fn read_into_slice<T: Pod>(&self, address: Address, slice: &mut [T]) -> Result<(), ()> {
        self.read_into_buf(address, bytemuck::cast_slice_mut(slice))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Address(pub u64);

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum PointerKind {
    U8 = 0,
    U16 = 1,
    U32 = 2,
    U64 = 3,
    I8 = 4,
    I16 = 5,
    I32 = 6,
    I64 = 7,
    F32 = 8,
    F64 = 9,
    String = 10,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum State {
    Old,
    Current,
}

pub fn start() {
    unsafe { sys::start() }
}

pub fn split() {
    unsafe { sys::split() }
}

pub fn reset() {
    unsafe { sys::reset() }
}

pub fn pause_game_time() {
    unsafe { sys::pause_game_time() }
}

pub fn resume_game_time() {
    unsafe { sys::resume_game_time() }
}

pub fn is_game_time_paused() -> i32 {
    unsafe { sys::is_game_time_paused() }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimerState {
    NotRunning,
    Running,
    Finished,
}

pub fn timer_state() -> TimerState {
    unsafe {
        match sys::get_timer_state() {
            0 => TimerState::NotRunning,
            1 => TimerState::Running,
            2 => TimerState::Finished,
            _ => core::hint::unreachable_unchecked(),
        }
    }
}

// pub fn set_process_name(module: &str) {
//     unsafe {
//         sys::set_process_name(module.as_ptr() as *const u8, module.len());
//     }
// }

// pub fn push_pointer_path(module: &str, offsets: &[i64], kind: PointerKind) {
//     unsafe {
//         let id = sys::push_pointer_path(module.as_ptr() as *const u8, module.len(), kind);
//         for &offset in offsets {
//             sys::push_offset(id, offset);
//         }
//     }
// }

// pub fn get_u8(pointer_path_id: usize, current: State) -> u8 {
//     unsafe { sys::get_u8(pointer_path_id, current) }
// }

// pub fn get_u16(pointer_path_id: usize, current: State) -> u16 {
//     unsafe { sys::get_u16(pointer_path_id, current) }
// }

// pub fn get_u32(pointer_path_id: usize, current: State) -> u32 {
//     unsafe { sys::get_u32(pointer_path_id, current) }
// }

// pub fn get_u64(pointer_path_id: usize, current: State) -> u64 {
//     unsafe { sys::get_u64(pointer_path_id, current) }
// }

// pub fn get_i8(pointer_path_id: usize, current: State) -> i8 {
//     unsafe { sys::get_i8(pointer_path_id, current) }
// }

// pub fn get_i16(pointer_path_id: usize, current: State) -> i16 {
//     unsafe { sys::get_i16(pointer_path_id, current) }
// }

// pub fn get_i32(pointer_path_id: usize, current: State) -> i32 {
//     unsafe { sys::get_i32(pointer_path_id, current) }
// }

// pub fn get_i64(pointer_path_id: usize, current: State) -> i64 {
//     unsafe { sys::get_i64(pointer_path_id, current) }
// }

// pub fn get_f32(pointer_path_id: usize, current: State) -> f32 {
//     unsafe { sys::get_f32(pointer_path_id, current) }
// }

// pub fn get_f64(pointer_path_id: usize, current: State) -> f64 {
//     unsafe { sys::get_f64(pointer_path_id, current) }
// }

// pub fn scan_signature(signature: &str) -> Option<Address> {
//     let address = unsafe { sys::scan_signature(signature.as_ptr(), signature.len()) };
//     if address.0 == 0 {
//         None
//     } else {
//         Some(address)
//     }
// }

pub fn set_tick_rate(ticks_per_second: f64) {
    unsafe { sys::set_tick_rate(ticks_per_second) }
}

pub fn print_message(text: &str) {
    unsafe { sys::print_message(text.as_ptr(), text.len()) }
}

// pub fn read_into_buf(address: Address, buf: &mut [u8]) -> bool {
//     unsafe { sys::read_into_buf(address, buf.as_mut_ptr(), buf.len()) }
// }

pub fn set_variable(key: &str, value: &str) {
    unsafe { sys::set_variable(key.as_ptr(), key.len(), value.as_ptr(), value.len()) }
}

// pub unsafe fn read_val<T>(address: Address) -> T {
//     let mut val = mem::MaybeUninit::<T>::uninit();
//     sys::read_into_buf(address, val.as_mut_ptr().cast(), mem::size_of::<T>());
//     val.assume_init()
// }

pub trait ASLState
where
    Self: Sized,
{
    fn get() -> (Self, Self);
}
