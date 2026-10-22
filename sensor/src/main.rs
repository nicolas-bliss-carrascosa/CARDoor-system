// Origional template taken from https://github.com/flipperzero-rs/flipperzero-template/tree/main 
#![no_main]
#![no_std]

// Required for panic handler
extern crate alloc;
extern crate flipperzero_alloc;
extern crate flipperzero_rt;

use alloc::format;
use alloc::string::String;
use flipperzero::dialogs::alert;
use core::ffi::{c_void, CStr};
use core::sync::atomic::{AtomicBool, Ordering};
use flipperzero_sys as sys;
use flipperzero_rt::{entry, manifest};

// Define the FAP Manifest
manifest!(
    name = "Card Sensor",
    app_version = 1,
    has_icon = true,
    icon = "rustacean-10x10.icon",
);

// Define the entry function
entry!(main);

// Global variables. Importent bbecause this lib requires constents for some reason
const RECORD_GUI: &[u8] = b"gui\0";
const PROTOCOL_COUNT: usize = 24;

static READ_SUCCESS: AtomicBool = AtomicBool::new(false);
static SHOULD_EXIT: AtomicBool = AtomicBool::new(false);

// C-compatible callback passed to the lfrfid_worker
unsafe extern "C" fn rfid_callback(event: sys::LFRFIDWorkerReadResult, _count:i32 ,_context: *mut c_void) {
    if event == sys::LFRFIDWorkerReadDone {
        READ_SUCCESS.store(true, Ordering::SeqCst);
    }
}
unsafe extern "C" fn input_callback(event: *mut sys::InputEvent, _context: *mut c_void) {
    if !event.is_null() {
        let ev = &*event;
        if ev.key == sys::InputKeyBack && (ev.type_ == sys::InputTypeShort || ev.type_ == sys::InputTypePress) {
            SHOULD_EXIT.store(true, Ordering::SeqCst);
        }
    }
}

// Draw callback to render UI state and dismiss the loader hourglass
unsafe extern "C" fn render_callback(canvas: *mut sys::Canvas, _context: *mut c_void) {
    unsafe {
        sys::canvas_clear(canvas);
        
        // Header
        sys::canvas_set_font(canvas, sys::FontPrimary);
        sys::canvas_draw_str(
            canvas,
            10,
            20,
            b"Scanning RFID...\0".as_ptr() as *const _,
        );

        // Subtitle instructions
        sys::canvas_set_font(canvas, sys::FontSecondary);
        sys::canvas_draw_str(
            canvas,
            10,
            38,
            b"Hold card to back of device\0".as_ptr() as *const _,
        );
        sys::canvas_draw_str(
            canvas,
            10,
            52,
            b"Press [Back] to cancel\0".as_ptr() as *const _,
        );
    }
}

// Entry point
fn main(_args: Option<&CStr>) -> i32 {
    alert("Start reading rfid");

    READ_SUCCESS.store(false, Ordering::SeqCst);
    SHOULD_EXIT.store(false, Ordering::SeqCst);
    unsafe {
        // Gui sections
        let gui = sys::furi_record_open(RECORD_GUI.as_ptr()) as *mut sys::Gui;
        let view_port = sys::view_port_alloc();
        sys::view_port_draw_callback_set(
            view_port,
            Some(render_callback),
            core::ptr::null_mut(),
        );
        sys::view_port_input_callback_set(
            view_port,
            Some(input_callback),
            core::ptr::null_mut(),
        );
        sys::gui_add_view_port(gui, view_port, sys::GuiLayerFullscreen);

        // Allocate protocol dictionary and make RFID worker
        let dict = sys::protocol_dict_alloc(
            sys::lfrfid_protocols.as_ptr(),
            PROTOCOL_COUNT,
        );
        let worker = sys::lfrfid_worker_alloc(dict);

        // Start workers
        sys::lfrfid_worker_start_thread(worker);
        sys::lfrfid_worker_read_start(
            worker,
            sys::LFRFIDWorkerReadTypeAuto,
            Some(rfid_callback),
            core::ptr::null_mut(),
        );

        // 6. Polling loop with UI active
        while !READ_SUCCESS.load(Ordering::SeqCst) && !SHOULD_EXIT.load(Ordering::SeqCst) {
            sys::furi_delay_ms(50);
        }

        // 7. Stop worker hardware
        sys::lfrfid_worker_stop(worker);
        sys::lfrfid_worker_stop_thread(worker);

        // 8. Extract payload if read succeeded
        let mut display_text = String::new();
        let was_scanned = READ_SUCCESS.load(Ordering::SeqCst);

        if was_scanned {
            display_text.push_str("Tag Detected!\n");
            for i in 0..PROTOCOL_COUNT {
                let data_size = sys::protocol_dict_get_data_size(dict, i);
                if data_size > 0 {
                    let name_ptr = sys::protocol_dict_get_name(dict, i);
                    if !name_ptr.is_null() {
                        let name = CStr::from_ptr(name_ptr).to_str().unwrap_or("Unknown");

                        let mut data_buf = [0u8; 32];
                        sys::protocol_dict_get_data(
                            dict,
                            i,
                            data_buf.as_mut_ptr(),
                            data_buf.len(),
                        );

                        display_text.push_str(&format!("Proto: {}\nUID: ", name));
                        for byte in &data_buf[..data_size] {
                            display_text.push_str(&format!("{:02X}", byte));
                        }
                        break;
                    }
                }
            }
        }

        // Clean up C workers
        sys::lfrfid_worker_free(worker);
        sys::protocol_dict_free(dict);

        // Detach ViewPort and close GUI handle
        sys::gui_remove_view_port(gui, view_port);
        sys::view_port_free(view_port);
        sys::furi_record_close(RECORD_GUI.as_ptr());
        if was_scanned {
            alert(&display_text);
        }
    }
    0
}