use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use std::mem::size_of;

use winapi::ctypes::c_void;
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HWND, HBITMAP, RECT, POINT, SIZE, HDC, HPEN};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::{
    CreateSolidBrush, CreateCompatibleDC, BITMAPINFOHEADER, BI_RGB, BITMAPINFO, RGBQUAD, SelectObject, BLENDFUNCTION,
    AC_SRC_OVER, AC_SRC_ALPHA, DeleteObject, DeleteDC, GdiFlush, MoveToEx, LineTo, HGDI_ERROR, CreatePen, PS_SOLID, CreateCompatibleBitmap, GetDIBits, DIB_RGB_COLORS
};
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, LoadCursorW, RegisterClassExW, ShowWindow, WNDCLASSEXW, CS_HREDRAW, CS_VREDRAW, WM_DESTROY, IDC_ARROW, SW_SHOW,
    CW_USEDEFAULT, WS_EX_LAYERED, WS_EX_TRANSPARENT, WS_EX_TOPMOST, WS_MAXIMIZE, EnumWindows, GetWindowTextW, PostQuitMessage, UpdateLayeredWindow,
    GetDC, ULW_ALPHA, ReleaseDC, PrintWindow, GetClientRect, PW_RENDERFULLCONTENT, OpenClipboard, SetClipboardData, EmptyClipboard, CloseClipboard, CF_BITMAP, FillRect
};

use crate::bitmap::RGBA;

// ###############################
// ############ Misc #############
// ###############################

/// Run a basic message loop for a given window handle
#[macro_export]
macro_rules! WindowsMessageLoop {
    ($handle: ident $(,$inner: tt)?) => {
        use winapi::um::winuser::{
            MSG, GetMessageW, TranslateMessage, DispatchMessageW
        };
        use winapi::shared::windef::POINT;

        let mut msg = MSG {
            hwnd: $handle,
            message: 0,
            wParam: 0,
            lParam: 0,
            time: 0,
            pt: POINT {x: 0, y: 0},
        };

        unsafe {
            while GetMessageW(&mut msg, $handle, 0, 0) > 0 {
                $($inner)?
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    };
}


fn to_wstring(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(once(0))
        .collect()
}

pub fn window_dimensions(hwnd: HWND) -> Result<(u32, u32), u32> {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    let result = unsafe { GetClientRect(hwnd, &mut rect) };

    if result == 0 {
        return Err(unsafe { GetLastError() })
    }

    let width = (rect.right - rect.left).unsigned_abs();
    let height = (rect.top - rect.bottom).unsigned_abs();

    Ok((width, height))
}

pub unsafe fn bitmap_to_clipboard(bitmap: HBITMAP) -> Result<(), u32> {
    if OpenClipboard(std::ptr::null_mut()) == 0 {
        return Err(GetLastError())
    }
    if EmptyClipboard() == 0 {
        CloseClipboard();
        return Err(GetLastError())
    }
    if SetClipboardData(CF_BITMAP, bitmap as *mut c_void).is_null() {
        CloseClipboard();
        return Err(GetLastError())
    }
    if CloseClipboard() == 0 {
        return Err(GetLastError())
    }

    Ok(())
}

pub unsafe fn bitmap_bits_to_buffer(hwnd: HWND, bitmap: HBITMAP, width: u32, height: u32, buffer: *mut u32) -> Result<(), u32> {
    let hdc = GetDC(hwnd);
    GetDIBits(hdc, bitmap, 0, height, buffer as *mut c_void, &mut create_bitmap_info(create_bitmap_header(width, height)), DIB_RGB_COLORS);
    ReleaseDC(hwnd, hdc);
    Ok(())
}

// ###############################
// #### Window Initialisation ####
// ###############################

pub unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
        }
        _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
    }
    return 0;
}

pub fn create_window() -> HWND {
    let app_name = to_wstring("Shellshock Tracer");

    let h_instance = unsafe { GetModuleHandleW(null_mut()) };

    let wnd_class = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: h_instance,
        hIcon: null_mut(),
        hCursor: unsafe { LoadCursorW(h_instance, IDC_ARROW) },
        hbrBackground: unsafe { CreateSolidBrush(0) },
        lpszMenuName: null_mut(),
        lpszClassName: app_name.as_ptr(),
        hIconSm: null_mut(),
    };

    let class_atom = unsafe { RegisterClassExW(&wnd_class) };

    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TRANSPARENT,
            class_atom as *const u16,
            app_name.as_ptr(),
            WS_MAXIMIZE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            null_mut(),
            null_mut(),
            h_instance,
            null_mut(),
        )
    };

    if hwnd.is_null() {
        panic!("Failed to create window.");
    }

    unsafe { ShowWindow(hwnd, SW_SHOW) };

    hwnd
}


// ###################################
// #### Shellshock handle finding ####
// ###################################
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
    // each char in string is 8 bits, but windows uses 16 bit chars, therefore double the capacity
    let mut buffer = Vec::with_capacity(100);
    let written = GetWindowTextW(hwnd, buffer.as_mut_ptr(), 100);
    buffer.set_len(written as usize);

    // ShellShock in u16 chars
    let search_buffer = [83, 104, 101, 108, 108, 83, 104, 111, 99, 107];

    for window in buffer.windows(10) {
        if search_buffer.as_slice() == window {
            let pointer = lparam as *mut HWND;
            *pointer = hwnd;
            return 0
        }
    }
    1
}

pub fn get_shellshock_window() -> Option<HWND> {
    let mut shellshock_window_hwnd: HWND = null_mut();
    let handle_ptr: *mut HWND = &mut shellshock_window_hwnd;

    let _result = unsafe { EnumWindows(Some(enum_windows_proc), handle_ptr as isize) };

    if handle_ptr.is_null() {
        None
    } else {
        Some(shellshock_window_hwnd)
    }
}


// ###################################
// ######### Window Drawing ##########
// ###################################
unsafe fn draw_cleanup(hwnd: HWND, hdc: HDC, mem_hdc: HDC, old: *mut c_void) -> Result<(), u32> {
    let mut return_result = Ok(());

    let result =  SelectObject(mem_hdc, old);
    if result.is_null() || result == HGDI_ERROR {
        return_result = Err(GetLastError())
    }

    if DeleteDC(mem_hdc) == 0 {
        return_result = Err(GetLastError())
    }
    
    if ReleaseDC(hwnd, hdc) == 0 {
        return_result = Err(GetLastError())
    }

    return_result
}

pub unsafe fn create_pen(width: u32, color: RGBA) -> HPEN {
    CreatePen(PS_SOLID as i32, width as i32, color.as_colorref())
}

pub fn create_bitmap_header(width: u32, height: u32) -> BITMAPINFOHEADER {
    BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width as i32,
        biHeight: height as i32,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB,
        biSizeImage: width*height*4,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
        biClrUsed: 0,
        biClrImportant: 0,
    }
}

pub fn create_bitmap_info(header: BITMAPINFOHEADER) -> BITMAPINFO {
    BITMAPINFO {
        bmiHeader: header,
        bmiColors: [
            RGBQUAD { rgbBlue: 0, rgbGreen: 0, rgbRed: 0, rgbReserved: 0 }
        ]
    }
}  

pub unsafe fn create_dibitmap(hwnd: HWND, dimensions: (u32, u32), color: RGBA) -> Result<HBITMAP, u32> {
    // everything is cleaned up after
    let hdc = GetDC(hwnd);
    let bitmap = CreateCompatibleBitmap(hdc, dimensions.0 as i32, dimensions.1 as i32);

    let mem_hdc = CreateCompatibleDC(hdc);
    let old = SelectObject(mem_hdc, bitmap as *mut c_void);

    let solid = CreateSolidBrush(color.as_colorref());
    FillRect(mem_hdc, &RECT {left: 0, top: 0, right: dimensions.0 as i32, bottom: dimensions.1 as i32}, solid);
    DeleteObject(solid as *mut c_void);

    if GdiFlush() == 0 {
        draw_cleanup(hwnd, hdc, mem_hdc, old)?;
        return Err(GetLastError())
    }

    draw_cleanup(hwnd, hdc, mem_hdc, old)?;

    if bitmap.is_null() {
        Err(GetLastError())
    } else {
        Ok(bitmap)
    }
}

#[derive(Debug)]
pub enum DrawError {
    UpdatingLayeredWindow(u32),
    Cleanup(u32)
}

pub unsafe fn draw_bitmap(hwnd: HWND, dibitmap: HBITMAP, width: u32, height: u32) -> Result<(), DrawError> {
    let hdc = GetDC(hwnd);
    let mem_hdc = CreateCompatibleDC(hdc);
    let old = SelectObject(mem_hdc, dibitmap as *mut c_void);

    let mut blend = BLENDFUNCTION {
        BlendOp: AC_SRC_OVER,
        BlendFlags: 0,
        SourceConstantAlpha: 255,
        AlphaFormat: AC_SRC_ALPHA,
    };

    let result = UpdateLayeredWindow(
        hwnd,
        null_mut(),
        null_mut(),
        &mut SIZE {cx: width as i32, cy: height as i32},
        mem_hdc,
        &mut POINT {x: 0, y: 0},
        0,
        &mut blend,
        ULW_ALPHA
    );
    if result == 0 {
        let _ = draw_cleanup(hwnd, hdc, mem_hdc, old);
        return Err(DrawError::UpdatingLayeredWindow(GetLastError()))
    }

   draw_cleanup(hwnd, hdc, mem_hdc, old).map_err(|code| DrawError::Cleanup(code))?;

    Ok(())
}

/// Returned error is a windows error code. If there is an error in drawing and in cleanup, the error code is the cleanup error code.
pub unsafe fn draw_line(hwnd: HWND, dibitmap: HBITMAP, pen: HPEN, from: (i32, i32), to: (i32, i32)) -> Result<(), u32> {
    let hdc = GetDC(hwnd);
    let mem_hdc = CreateCompatibleDC(hdc);

    let old_bmap = SelectObject(mem_hdc, dibitmap as *mut c_void);
    let old_pen = SelectObject(mem_hdc, pen as *mut c_void);

    let result = MoveToEx(mem_hdc, from.0, from.1, null_mut());
    if result == 0 {
        draw_cleanup(hwnd, hdc, mem_hdc, old_bmap)?;
        return Err(GetLastError())
    }

    let result = LineTo(mem_hdc, to.0, to.1);
    if result == 0 {
        // don't need to handle any error as cleanup must run, and an error will be returned anyway
        let _result = SelectObject(mem_hdc, old_pen);
        draw_cleanup(hwnd, hdc, mem_hdc, old_bmap)?;
        return Err(GetLastError())
    }

    let result = SelectObject(mem_hdc, old_pen);
    if result.is_null() || result == HGDI_ERROR {
        draw_cleanup(hwnd, hdc, mem_hdc, old_bmap)?;
        return Err(GetLastError());
    }

    draw_cleanup(hwnd, hdc, mem_hdc, old_bmap)?;

    Ok(())
}

/// Uses a curve function that takes an x value and returns a y value.
pub unsafe fn draw_dotted_curve<F: FnMut(i32) -> i32>(hwnd: HWND, dibitmap: HBITMAP, pen: HPEN, start_x: i32, end_x: i32, dot_length: u32, mut curve: F) -> Result<(), u32> {
    let mut solid_part = true;
    let mut temp_start = (start_x, curve(start_x));

    for x in (start_x+1)..=end_x {
        let y = curve(x);
        let square_sum = (x-temp_start.0).pow(2) + (y-temp_start.1).pow(2);
        let current_line_length = (square_sum as f32).sqrt() as i32;

        if current_line_length >= dot_length as i32 {
            if solid_part {
                draw_line(hwnd, dibitmap, pen, temp_start, (x, y))?;
            }
            solid_part = !solid_part;
            temp_start = (x, y);
        }
    }

    Ok(())
}

/// Uses a curve function that takes a parameter t, the distance along the line, and returns a (x, y) coordinate.
/// The curve is stopped when x < 0 or x > max_x, or y > max_y.
pub unsafe fn draw_dotted_parametric_curve<F: FnMut(i32) -> (i32, i32)>(hwnd: HWND, dibitmap: HBITMAP, pen: HPEN, max_x: i32, max_y: i32, dot_length: u32, mut curve: F) -> Result<(), u32> {
    let mut solid_part = true;
    let mut t = 0;
    let mut temp_start = curve(t);

    loop {
        t += 1;
        let current = curve(t);

        if current.0 > max_x || current.0 <= 0 || current.1 > max_y {
            break
        }

        let square_sum = (current.0-temp_start.0).pow(2) + (current.1-temp_start.1).pow(2);
        let current_line_length = (square_sum as f32).sqrt() as i32;

        if current_line_length >= dot_length as i32 {
            if solid_part {
                draw_line(hwnd, dibitmap, pen, temp_start, current)?;
            }
            solid_part = !solid_part;
            temp_start = current;
        }
    }

    Ok(())
}

pub unsafe fn object_cleanup(bitmap: HBITMAP, pen: HPEN) {
    DeleteObject(bitmap as *mut c_void);
    DeleteObject(pen as *mut c_void);
}


// ###################################
// ######### Screen Capture ##########
// ###################################

pub unsafe fn screen_capture(hwnd: HWND) -> Result<HBITMAP, u32> {
    let dimensions = window_dimensions(hwnd)?;
    
    let hdc = GetDC(hwnd);
    let mem_hdc = CreateCompatibleDC(hdc);
    let bitmap = CreateCompatibleBitmap(hdc, dimensions.0 as i32, dimensions.1 as i32);
    let old = SelectObject(mem_hdc, bitmap as *mut c_void);
    
    let result = PrintWindow(hwnd, mem_hdc, PW_RENDERFULLCONTENT);
    
    if result == 0 {
        draw_cleanup(hwnd, hdc, mem_hdc, old)?;
        DeleteObject(bitmap as *mut c_void);
        return Err(GetLastError())
    }
    
    draw_cleanup(hwnd, hdc, mem_hdc, old)?;
    Ok(bitmap)
}
