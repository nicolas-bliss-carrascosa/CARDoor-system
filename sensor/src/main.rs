#![no_main]
#![no_std]

extern crate alloc;
extern crate flipperzero_alloc;
extern crate flipperzero_rt;

use alloc::format;
use alloc::string::String;
use core::cell::UnsafeCell;
use core::ffi::{c_void, CStr};
use core::sync::atomic::{AtomicBool, Ordering};

use flipperzero_rt::{entry, manifest};
use flipperzero_sys as sys;

manifest!(
    name = "Card Sensor",
    app_version = 1,
    has_icon = false,
);

entry!(main);

const RECORD_GUI: &[u8] = b"gui\0";
const RECORD_STORAGE: &[u8] = b"storage\0";
const SCAN_DIR_PARENT: &[u8] = b"/ext/apps_data\0";
const SCAN_DIR: &[u8] = b"/ext/apps_data/card_sensor\0";
const SCAN_FILE: &[u8] = b"/ext/apps_data/card_sensor/scan.txt\0";
const PROTOCOL_COUNT: usize = 24;

static READ_SUCCESS: AtomicBool = AtomicBool::new(false);
static SHOULD_EXIT: AtomicBool = AtomicBool::new(false);
static HAS_LAST_SCAN: AtomicBool = AtomicBool::new(false);

struct SyncBuffer(UnsafeCell<[u8; 48]>);
unsafe impl Sync for SyncBuffer {}

impl SyncBuffer {
    const fn new() -> Self {
        Self(UnsafeCell::new([0u8; 48]))
    }

    unsafe fn write_str(&self, prefix: &str, text: &str) {
        let buf = &mut *self.0.get();
        buf.fill(0);
        let mut idx = 0;
        for &b in prefix.as_bytes().iter().chain(text.as_bytes().iter()) {
            if idx >= buf.len() - 1 {
                break;
            }
            buf[idx] = b;
            idx += 1;
        }
    }

    unsafe fn as_ptr(&self) -> *const core::ffi::c_char {
        (*self.0.get()).as_ptr() as *const _
    }
}

static LAST_PROTO_LINE: SyncBuffer = SyncBuffer::new();
static LAST_UID_LINE: SyncBuffer = SyncBuffer::new();

unsafe extern "C" fn rfid_callback(
    event: sys::LFRFIDWorkerReadResult,
    _count: i32,
    _context: *mut c_void,
) {
    if event == sys::LFRFIDWorkerReadDone {
        READ_SUCCESS.store(true, Ordering::SeqCst);
    }
}

unsafe extern "C" fn render_callback(canvas: *mut sys::Canvas, _context: *mut c_void) {
    unsafe { 
        sys::canvas_clear(canvas);

        sys::canvas_set_font(canvas, sys::FontPrimary);
        sys::canvas_draw_str(
            canvas,
            8,
            14,
            b"Scanning RFID...\0".as_ptr() as *const _,
        );

        sys::canvas_set_font(canvas, sys::FontSecondary);
        if HAS_LAST_SCAN.load(Ordering::Acquire) {
            sys::canvas_draw_str(canvas, 8, 30, LAST_PROTO_LINE.as_ptr());
            sys::canvas_draw_str(canvas, 8, 43, LAST_UID_LINE.as_ptr());
            sys::canvas_draw_str(
                canvas,
                8,
                58,
                b"Ready for next | [Back] exit\0".as_ptr() as *const _,
            );
        } else {
            sys::canvas_draw_str(
                canvas,
                8,
                32,
                b"Hold card to back of device\0".as_ptr() as *const _,
            );
            sys::canvas_draw_str(
                canvas,
                8,
                48,
                b"Press [Back] to exit\0".as_ptr() as *const _,
            );
        }
    };
}

unsafe extern "C" fn input_callback(event: *mut sys::InputEvent, _context: *mut c_void) {
    unsafe { 
        if !event.is_null() {
            let ev = &*event;
            if ev.key == sys::InputKeyBack
                && (ev.type_ == sys::InputTypeShort || ev.type_ == sys::InputTypePress)
            {
                SHOULD_EXIT.store(true, Ordering::SeqCst);
            }
        }
}
}

unsafe fn beep_confirm() {
    unsafe { 
        if sys::furi_hal_speaker_acquire(1000) {
            sys::furi_hal_speaker_start(1000.0, 1.0);
            sys::furi_delay_ms(100);
            sys::furi_hal_speaker_stop();
            sys::furi_hal_speaker_release();
        }
    }
}

unsafe fn write_scan_file(storage: *mut sys::Storage, payload: &str) {
    unsafe { 
        sys::storage_simply_mkdir(storage, SCAN_DIR_PARENT.as_ptr() as *const _);
        sys::storage_simply_mkdir(storage, SCAN_DIR.as_ptr() as *const _);

        let file = sys::storage_file_alloc(storage);
        if sys::storage_file_open(
            file,
            SCAN_FILE.as_ptr() as *const _,
            sys::FSAM_WRITE,
            sys::FSOM_CREATE_ALWAYS,
        ) {
            sys::storage_file_write(file, payload.as_ptr() as *const c_void, payload.len());
            sys::storage_file_close(file);
        }
        sys::storage_file_free(file);
    }
}

fn main(_args: Option<&CStr>) -> i32 {
    READ_SUCCESS.store(false, Ordering::SeqCst);
    SHOULD_EXIT.store(false, Ordering::SeqCst);
    HAS_LAST_SCAN.store(false, Ordering::SeqCst);

    unsafe {
        let gui = sys::furi_record_open(RECORD_GUI.as_ptr()) as *mut sys::Gui;
        let storage = sys::furi_record_open(RECORD_STORAGE.as_ptr()) as *mut sys::Storage;

        let view_port = sys::view_port_alloc();
        sys::view_port_draw_callback_set(view_port, Some(render_callback), core::ptr::null_mut());
        sys::view_port_input_callback_set(view_port, Some(input_callback), core::ptr::null_mut());
        sys::gui_add_view_port(gui, view_port, sys::GuiLayerFullscreen);

        // Continuous scan loop until Back is pressed
        while !SHOULD_EXIT.load(Ordering::SeqCst) {
            READ_SUCCESS.store(false, Ordering::SeqCst);

            let dict = sys::protocol_dict_alloc(sys::lfrfid_protocols.as_ptr(), PROTOCOL_COUNT);
            let worker = sys::lfrfid_worker_alloc(dict);

            sys::lfrfid_worker_start_thread(worker);
            sys::lfrfid_worker_read_start(
                worker,
                sys::LFRFIDWorkerReadTypeAuto,
                Some(rfid_callback),
                core::ptr::null_mut(),
            );

            while !READ_SUCCESS.load(Ordering::SeqCst) && !SHOULD_EXIT.load(Ordering::SeqCst) {
                sys::furi_delay_ms(50);
            }

            sys::lfrfid_worker_stop(worker);
            sys::lfrfid_worker_stop_thread(worker);

            if READ_SUCCESS.load(Ordering::SeqCst) && !SHOULD_EXIT.load(Ordering::SeqCst) {
                for i in 0..PROTOCOL_COUNT {
                    let data_size = sys::protocol_dict_get_data_size(dict, i);
                    if data_size > 0 {
                        let mut data_buf = [0u8; 32];
                        sys::protocol_dict_get_data(
                            dict,
                            i,
                            data_buf.as_mut_ptr(),
                            data_buf.len(),
                        );

                        if data_buf[..data_size].iter().any(|&b| b != 0) {
                            let name_ptr = sys::protocol_dict_get_name(dict, i);
                            let name = if !name_ptr.is_null() {
                                CStr::from_ptr(name_ptr).to_str().unwrap_or("Unknown")
                            } else {
                                "Unknown"
                            };

                            let mut uid_hex = String::new();
                            for byte in &data_buf[..data_size] {
                                uid_hex.push_str(&format!("{:02X}", byte));
                            }

                            let payload = format!("{}|{}", name, uid_hex);
                            write_scan_file(storage, &payload);

                            HAS_LAST_SCAN.store(false, Ordering::Release);
                            LAST_PROTO_LINE.write_str("Proto: ", name);
                            LAST_UID_LINE.write_str("UID: ", &uid_hex);
                            HAS_LAST_SCAN.store(true, Ordering::Release);
                            sys::view_port_update(view_port);

                            beep_confirm();

                            // Debounce delay before arming the next scan cycle
                            for _ in 0..12 {
                                if SHOULD_EXIT.load(Ordering::SeqCst) {
                                    break;
                                }
                                sys::furi_delay_ms(100);
                            }
                            break;
                        }
                    }
                }
            }

            sys::lfrfid_worker_free(worker);
            sys::protocol_dict_free(dict);
        }

        sys::gui_remove_view_port(gui, view_port);
        sys::view_port_free(view_port);
        sys::furi_record_close(RECORD_STORAGE.as_ptr());
        sys::furi_record_close(RECORD_GUI.as_ptr());
    }

    0
}