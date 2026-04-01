// Extract the icon of a Windows executable and cache it as a PNG file.
//
// The public entry-point is [`extract_icon_to_cache`].  It is safe to call
// from multiple threads; each call is self-contained and the only shared
// state is the on-disk cache directory.

use image::RgbaImage;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::os::windows::ffi::OsStrExt as _;
use std::path::{Path, PathBuf};
use tracing::warn;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC,
    DeleteObject, GetDIBits, GetObjectW, HGDIOBJ, SelectObject,
};
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use windows::Win32::UI::Shell::{SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGetFileInfoW};
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO};
use windows::core::PCWSTR;

/// Width/height of the icon bitmap we request and write to PNG.
const ICON_SIZE: u32 = 32;

// Public API

/// Extract the icon for `exe_path` and write it as a PNG into `cache_dir`.
///
/// The output filename is derived from a hash of `exe_path`, so repeated calls
/// for the same executable return the cached file immediately.
///
/// Returns the full path of the cached PNG, or `None` if extraction fails.
pub fn extract_icon_to_cache(exe_path: &str, cache_dir: &Path) -> Option<PathBuf> {
    // Stable filename: hex of DefaultHasher over the source path string.
    let mut hasher = DefaultHasher::new();
    exe_path.hash(&mut hasher);
    let hash = hasher.finish();
    let cache_path = cache_dir.join(format!("{hash:016x}.png"));

    // Fast path: already cached.
    if cache_path.exists() {
        return Some(cache_path);
    }

    if let Err(e) = std::fs::create_dir_all(cache_dir) {
        warn!("failed to create icon cache dir: {e}");
        return None;
    }

    let wide: Vec<u16> = std::ffi::OsStr::new(exe_path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // SAFETY: we pass a valid null-terminated wide string; all GDI handles are
    // cleaned up before this function returns.
    unsafe { extract_and_save(PCWSTR(wide.as_ptr()), &cache_path) }
}

/// Call `SHGetFileInfoW` to obtain an `HICON`, then render it to `output`.
///
/// # Safety
/// Caller must ensure `path` points to a valid null-terminated UTF-16 string.
unsafe fn extract_and_save(path: PCWSTR, output: &Path) -> Option<PathBuf> {
    let mut shfi = unsafe { std::mem::zeroed::<SHFILEINFOW>() };

    let result = unsafe {
        SHGetFileInfoW(
            path,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };

    if result == 0 {
        warn!("SHGetFileInfoW failed (returned 0)");
        return None;
    }

    let hicon: HICON = shfi.hIcon;
    if hicon.is_invalid() {
        return None;
    }

    let result = unsafe { render_icon_to_png(hicon, output) };

    let _ = unsafe { DestroyIcon(hicon) };

    result
}

/// Use GDI to read the bitmap pixels from `hicon` and encode them as a PNG.
///
/// # Safety
/// `hicon` must be a valid icon handle obtained from the Shell API.
unsafe fn render_icon_to_png(hicon: HICON, output: &Path) -> Option<PathBuf> {
    // Obtain the color and mask bitmaps for this icon
    let mut icon_info = unsafe { std::mem::zeroed::<ICONINFO>() };
    if unsafe { GetIconInfo(hicon, &mut icon_info) }.is_err() {
        warn!("GetIconInfo failed");
        return None;
    }

    let hbm_color = icon_info.hbmColor;
    let hbm_mask = icon_info.hbmMask;

    // Helper to delete a GDI object, converting HBITMAP → HGDIOBJ.
    let delete_bm = |hbm: windows::Win32::Graphics::Gdi::HBITMAP| {
        if !hbm.is_invalid() {
            unsafe {
                let _ = DeleteObject(HGDIOBJ(hbm.0));
            }
        }
    };

    if hbm_color.is_invalid() {
        // Monochrome-only icon — skip it.
        delete_bm(hbm_mask);
        return None;
    }

    // Query the actual bitmap dimensions
    let mut bm = unsafe { std::mem::zeroed::<BITMAP>() };
    let got = unsafe {
        GetObjectW(
            HGDIOBJ(hbm_color.0),
            std::mem::size_of::<BITMAP>() as i32,
            Some(&mut bm as *mut _ as *mut _),
        )
    };

    let (width, height) = if got > 0 && bm.bmWidth > 0 && bm.bmHeight != 0 {
        (bm.bmWidth as u32, bm.bmHeight.unsigned_abs())
    } else {
        // Fallback to the constant if GetObject fails.
        (ICON_SIZE, ICON_SIZE)
    };

    // Create a temporary DC and read pixels with GetDIBits
    let hdc = unsafe { CreateCompatibleDC(None) };
    if hdc.is_invalid() {
        warn!("CreateCompatibleDC failed");
        delete_bm(hbm_color);
        delete_bm(hbm_mask);
        return None;
    }

    // Select the colour bitmap into the DC.
    let old_obj = unsafe { SelectObject(hdc, HGDIOBJ(hbm_color.0)) };

    // Describe the target DIB: 32 bpp, top-down (negative height).
    let mut bmi = unsafe { std::mem::zeroed::<BITMAPINFO>() };
    bmi.bmiHeader = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width as i32,
        biHeight: -(height as i32), // negative → top-down row order
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        biSizeImage: 0,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
        biClrUsed: 0,
        biClrImportant: 0,
    };

    let mut pixels = vec![0u8; (width * height * 4) as usize];

    let lines = unsafe {
        GetDIBits(
            hdc,
            hbm_color,
            0,
            height,
            Some(pixels.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        )
    };

    // Restore and release GDI resources
    unsafe {
        SelectObject(hdc, old_obj);
        let _ = DeleteDC(hdc);
    }
    delete_bm(hbm_color);
    delete_bm(hbm_mask);

    if lines == 0 {
        warn!("GetDIBits returned 0 scan lines");
        return None;
    }

    // BGRA → RGBA channel swap
    for chunk in pixels.chunks_exact_mut(4) {
        chunk.swap(0, 2); // B ↔ R
    }

    // Encode to PNG and write
    let img = RgbaImage::from_raw(width, height, pixels)?;
    if let Err(e) = img.save(output) {
        warn!("failed to save icon PNG to {}: {e}", output.display());
        return None;
    }

    Some(output.to_path_buf())
}
