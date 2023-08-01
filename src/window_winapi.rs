use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use std::mem::size_of;

use thiserror::Error;

use winapi::ctypes::c_void;
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HWND, HBITMAP, RECT, POINT, SIZE, HDC, HPEN, HBRUSH};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::{
    CreateSolidBrush, CreateCompatibleDC, BITMAPINFOHEADER, BI_RGB, BITMAPINFO, RGBQUAD, SelectObject, BLENDFUNCTION,
    AC_SRC_OVER, AC_SRC_ALPHA, DeleteObject, DeleteDC, GdiFlush, MoveToEx, LineTo, HGDI_ERROR, CreatePen, PS_SOLID, CreateCompatibleBitmap, GetDIBits, DIB_RGB_COLORS, GetStockObject, BLACK_BRUSH
};
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, LoadCursorW, RegisterClassExW, ShowWindow, WNDCLASSEXW, CS_HREDRAW, CS_VREDRAW, WM_DESTROY, IDC_ARROW, SW_SHOW,
    CW_USEDEFAULT, WS_EX_LAYERED, WS_EX_TRANSPARENT, WS_EX_TOPMOST, WS_MAXIMIZE, EnumWindows, GetWindowTextW, PostQuitMessage, UpdateLayeredWindow,
    GetDC, ULW_ALPHA, ReleaseDC, PrintWindow, PW_RENDERFULLCONTENT, OpenClipboard, SetClipboardData, EmptyClipboard, CloseClipboard, CF_BITMAP, FillRect, GetWindowRect
};

use crate::{Coordinate, Size};
use crate::bitmap::ARGB;

// ###############################
// ############ Misc #############
// ###############################

#[derive(Debug)]
pub enum WindowsErrorType {
    Other,
    DeleteMemoryDc,
    CreateMemoryDc,
    GetDc,
    ReleaseDc,
    CreateObject,
    DeleteObject,
    GetModuleHandle,
    RegisterWindowClass,
    CreateWindow,
    SelectObject,
    CreateBitmap,
    UpdateLayeredWindow
}

#[derive(Error, Debug)]
#[error("Error with the Windows API (code: {}, type: {:?})", self.code, self.error_type)]
pub struct WindowsError {
    pub code: u32,
    pub error_type: WindowsErrorType
}

impl From<u32> for WindowsError {
    fn from(value: u32) -> Self {
        Self { code: value, error_type: WindowsErrorType::Other }
    }
}

/// Run a basic message loop for a given window handle
#[macro_export]
macro_rules! WindowsMessageLoop {
    ($handle: ident, $loop_time: ident $(,$inner: tt)?) => {
        use std::thread;
        use std::ptr::null_mut;
        use winapi::um::winuser::{
            MSG, TranslateMessage, DispatchMessageW, PeekMessageW, PM_REMOVE, WM_QUIT
        };
        use winapi::shared::windef::POINT;

        let mut msg = MSG {
            hwnd: null_mut(),
            message: 0,
            wParam: 0,
            lParam: 0,
            time: 0,
            pt: POINT {x: 0, y: 0},
        };

        unsafe {
            'outer: loop {
                while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                    if msg.message == WM_QUIT {
                        break 'outer;
                    }
                }
                $($inner)?
                thread::sleep($loop_time);
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

pub unsafe fn window_dimensions(hwnd: HWND) -> Result<Size<u32>, WindowsError> {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };

    let result = unsafe { GetWindowRect(hwnd, &mut rect) };

    if result == 0 {
        return Err(unsafe { GetLastError().into() })
    }

    let width = (rect.right - rect.left).unsigned_abs();
    let height = (rect.top - rect.bottom).unsigned_abs();

    // win 10 has a 8 pixel border on every side
    Ok(Size(width - 16, height - 16))
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
    0
}

pub fn create_window() -> Result<HWND, WindowsError> {
    let app_name = to_wstring("Shellshock Tracer");

    let h_instance = unsafe { GetModuleHandleW(null_mut()) };

    if h_instance.is_null() {
        return unsafe { Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::GetModuleHandle }) }
    }

    let wnd_class = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: h_instance,
        hIcon: null_mut(),
        hCursor: unsafe { LoadCursorW(h_instance, IDC_ARROW) },
        hbrBackground: unsafe { GetStockObject(BLACK_BRUSH as i32) as HBRUSH },
        lpszMenuName: null_mut(),
        lpszClassName: app_name.as_ptr(),
        hIconSm: null_mut(),
    };

    let class_atom = unsafe { RegisterClassExW(&wnd_class) };

    if class_atom == 0 {
        return unsafe { Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::RegisterWindowClass }) }
    }

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
        return unsafe { Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::CreateWindow }) }
    }

    unsafe { ShowWindow(hwnd, SW_SHOW) };

    Ok(hwnd)
}

// ###################################
// #### Shellshock handle finding ####
// ###################################

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> i32 {
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

unsafe fn create_mem_dc(hwnd: HWND) -> Result<(HDC, HDC), WindowsError> {
    let hdc = GetDC(hwnd);
    if hdc.is_null() {
        return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::GetDc })
    }

    let mem_hdc = CreateCompatibleDC(hdc);
    if mem_hdc.is_null() {
        ReleaseDC(hwnd, hdc);
        return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::CreateMemoryDc })
    }

    Ok((hdc, mem_hdc))
}

unsafe fn draw_cleanup(hwnd: HWND, hdc: HDC, mem_hdc: HDC, old: *mut c_void) -> Result<(), WindowsError> {
    let mut return_result = Ok(());

    let result =  SelectObject(mem_hdc, old);
    if result.is_null() || result == HGDI_ERROR {
        return_result = Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::SelectObject })
    }

    if DeleteDC(mem_hdc) == 0 {
        return_result = Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::DeleteMemoryDc })
    }
    
    if ReleaseDC(hwnd, hdc) == 0 {
        return_result = Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::ReleaseDc })
    }

    return_result
}

pub unsafe fn create_pen(width: u32, color: ARGB) -> HPEN {
    CreatePen(PS_SOLID as i32, width as i32, color.as_colorref())
}

pub fn create_bitmap_header(dimensions: Size<u32>) -> BITMAPINFOHEADER {
    BITMAPINFOHEADER {
        biSize: size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: dimensions.0 as i32,
        biHeight: dimensions.1 as i32,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB,
        biSizeImage: dimensions.0*dimensions.1*4,
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

pub unsafe fn create_dibitmap(hwnd: HWND, dimensions: Size<u32>, color: ARGB) -> Result<HBITMAP, WindowsError> {
    let (hdc, mem_hdc) = create_mem_dc(hwnd)?;

    let bitmap = CreateCompatibleBitmap(hdc, dimensions.0 as i32, dimensions.1 as i32);

    if bitmap.is_null() {
        DeleteDC(mem_hdc);
        ReleaseDC(hwnd, hdc);
        return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::CreateBitmap })
    }

    let old = SelectObject(mem_hdc, bitmap as *mut c_void);
    if old.is_null() {
        DeleteObject(bitmap as *mut c_void);
        DeleteDC(mem_hdc);
        ReleaseDC(hwnd, hdc);
        return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::SelectObject });
    }

    let solid = CreateSolidBrush(color.as_colorref());
    if solid.is_null() {
        DeleteObject(bitmap as *mut c_void);
        let _ = draw_cleanup(hwnd, hdc, mem_hdc, old);
        return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::CreateObject })
    }

    let fill_result = FillRect(mem_hdc, &RECT {left: 0, top: 0, right: dimensions.0 as i32, bottom: dimensions.1 as i32}, solid);
    let delete_result = DeleteObject(solid as *mut c_void);
    let flush_result = GdiFlush();

    draw_cleanup(hwnd, hdc, mem_hdc, old)?;

    if fill_result == 0 { return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::Other }) }
    if delete_result == 0 { return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::DeleteObject }) }
    if flush_result == 0 { return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::Other }) }

    Ok(bitmap)
}

pub unsafe fn clear_bitmap(hwnd: HWND, dibitmap: HBITMAP, dimensions: Size<u32>) -> Result<(), WindowsError> {
    let (hdc, mem_hdc) = create_mem_dc(hwnd)?;

    let old = SelectObject(mem_hdc, dibitmap as *mut c_void);
    if old.is_null() {
        DeleteDC(mem_hdc);
        ReleaseDC(hwnd, hdc);
        return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::SelectObject });
    }

    let solid = CreateSolidBrush(0);
    if solid.is_null() {
        let _ = draw_cleanup(hwnd, hdc, mem_hdc, old);
        return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::CreateObject })
    }

    let fill_result = FillRect(mem_hdc, &RECT {left: 0, top: 0, right: dimensions.0 as i32, bottom: dimensions.1 as i32}, solid);
    let delete_result = DeleteObject(solid as *mut c_void);

    draw_cleanup(hwnd, hdc, mem_hdc, old)?;

    if fill_result == 0 { return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::Other }) }
    if delete_result == 0 { return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::DeleteObject }) }

    Ok(())
}

pub unsafe fn draw_bitmap(hwnd: HWND, dibitmap: HBITMAP, dimensions: Size<u32>) -> Result<(), WindowsError> {
    let (hdc, mem_hdc) = create_mem_dc(hwnd)?;

    let old = SelectObject(mem_hdc, dibitmap as *mut c_void);
    if old.is_null() {
        DeleteDC(mem_hdc);
        ReleaseDC(hwnd, hdc);
        return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::SelectObject });
    }

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
        &mut SIZE {cx: dimensions.0 as i32, cy: dimensions.1 as i32},
        mem_hdc,
        &mut POINT {x: 0, y: 0},
        0,
        &mut blend,
        ULW_ALPHA
    );

   draw_cleanup(hwnd, hdc, mem_hdc, old)?;

   if result == 0 {
       return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::UpdateLayeredWindow })
    }

    Ok(())
}

/// Coordinates relative to bottom-left
/// Returned error is a windows error code. If there is an error in drawing and in cleanup, the error code is the cleanup error code.
pub unsafe fn draw_line(hwnd: HWND, dibitmap: HBITMAP, dimensions: Size<u32>, pen: HPEN, from: Coordinate<i32>, to: Coordinate<i32>) -> Result<(), WindowsError> {
    // account for 8 pixel window border
    let from = (from.0+8, from.1-8);
    let to = (to.0+8, to.1-8);

    // coordinates are relative to bottom-left, so use height to flip it
    let height = dimensions.1 as i32;

    let (hdc, mem_hdc) = create_mem_dc(hwnd)?;

    let old_bmap = SelectObject(mem_hdc, dibitmap as *mut c_void);
    if old_bmap.is_null() {
        DeleteDC(mem_hdc);
        ReleaseDC(hwnd, hdc);
        return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::SelectObject })
    }

    let old_pen = SelectObject(mem_hdc, pen as *mut c_void);
    if old_pen.is_null() {
        let _ = draw_cleanup(hwnd, hdc, mem_hdc, old_bmap);
        return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::SelectObject })
    }

    let move_result = MoveToEx(mem_hdc, from.0, height-from.1, null_mut());
    let line_result = LineTo(mem_hdc, to.0, height-to.1);
    let select_result = SelectObject(mem_hdc, old_pen);

    draw_cleanup(hwnd, hdc, mem_hdc, old_bmap)?;

    if move_result == 0 || line_result == 0 {
        return Err(GetLastError().into())
    }

    if select_result.is_null() {
        return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::SelectObject });
    }

    Ok(())
}

/// Uses a curve function that takes a parameter t, the distance along the line, and returns a (x, y) coordinate.
/// The curve is stopped when x < 0 or x > max_x, or y > max_y.
pub unsafe fn draw_dotted_parametric_curve<F: FnMut(i32) -> Coordinate<i32>>(hwnd: HWND, dibitmap: HBITMAP, dimensions: Size<u32>, pen: HPEN, mut curve: F) -> Result<(), WindowsError> {
    /// The length of dotted lines drawn.
    const DOT_LENGTH: i32 = 4;

    let mut solid_part = true;
    let mut t = 0;
    let mut temp_start = curve(t);

    let (max_x, _max_y) = (dimensions.0 as i32, dimensions.1 as i32);

    loop {
        t += 1;
        let current = curve(t);

        if current.0 > max_x || current.0 <= 0 || current.1 <= 0 {
            break
        }

        let square_sum = (current.0-temp_start.0).pow(2) + (current.1-temp_start.1).pow(2);
        let current_line_length = (square_sum as f32).sqrt() as i32;

        if current_line_length >= DOT_LENGTH {
            if solid_part {
                draw_line(hwnd, dibitmap, dimensions, pen, temp_start, current)?;
            }
            solid_part = !solid_part;
            temp_start = current;
        }
    }

    Ok(())
}

pub unsafe fn bitmap_to_clipboard(bitmap: HBITMAP) -> Result<(), WindowsError> {
    if OpenClipboard(std::ptr::null_mut()) == 0 {
        return Err(GetLastError().into())
    }
    if EmptyClipboard() == 0 {
        CloseClipboard();
        return Err(GetLastError().into())
    }
    if SetClipboardData(CF_BITMAP, bitmap as *mut c_void).is_null() {
        CloseClipboard();
        return Err(GetLastError().into())
    }
    if CloseClipboard() == 0 {
        return Err(GetLastError().into())
    }

    Ok(())
}

pub unsafe fn bitmap_bits_to_buffer(hwnd: HWND, bitmap: HBITMAP, size: Size<u32>, buffer: *mut ARGB) -> Result<(), WindowsError> {
    let hdc = GetDC(hwnd);
    let result_scanlines = GetDIBits(hdc, bitmap, 0, size.1, buffer as *mut c_void, &mut create_bitmap_info(create_bitmap_header(size)), DIB_RGB_COLORS);

    if ReleaseDC(hwnd, hdc) == 0 {
        return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::ReleaseDc })
    }

    if result_scanlines == 0 {
        return Err(GetLastError().into())
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

pub unsafe fn screen_capture(hwnd: HWND) -> Result<HBITMAP, WindowsError> {
    let dimensions = window_dimensions(hwnd)?;
    
    let (hdc, mem_hdc) = create_mem_dc(hwnd)?;

    let bitmap = CreateCompatibleBitmap(hdc, dimensions.0 as i32, dimensions.1 as i32);
    if bitmap.is_null() {
        DeleteDC(mem_hdc);
        ReleaseDC(hwnd, hdc);
        return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::CreateBitmap })
    }

    let old = SelectObject(mem_hdc, bitmap as *mut c_void);
    if old.is_null() {
        DeleteObject(bitmap as *mut c_void);
        DeleteDC(mem_hdc);
        ReleaseDC(hwnd, hdc);
        return Err(WindowsError { code: GetLastError(), error_type: WindowsErrorType::SelectObject })
    }
    
    let result = PrintWindow(hwnd, mem_hdc, PW_RENDERFULLCONTENT);
    
    draw_cleanup(hwnd, hdc, mem_hdc, old)?;

    if result == 0 {
        DeleteObject(bitmap as *mut c_void);
        return Err(GetLastError().into())
    }

    Ok(bitmap)
}
