#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    clippy::approx_constant,
    clippy::useless_transmute,
    clippy::zero_sized_map_values,
    unnecessary_transmutes,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    unused_unsafe,
)]

pub use ffmpeg_sys::*;

mod raw {
    pub use ffmpeg_sys::*;
}

use std::path::Path;
use std::ffi::{CString, CStr};
use std::os::raw::c_int;
use std::ptr;

const AVERROR_EOF: i32 = -541_478_725;

pub struct VideoDecoder {
    format_context: *mut raw::AVFormatContext,
    codec_context: *mut raw::AVCodecContext,
    sws_context: *mut raw::SwsContext,
    video_stream_index: c_int,
    width: c_int,
    height: c_int,
    frame_rate: f64,
    current_frame: i32,
    frame: *mut raw::AVFrame,
    rgba_frame: *mut raw::AVFrame,
    packet: *mut raw::AVPacket,
    rgba_data: Vec<u8>,
    eof: bool,
}

impl VideoDecoder {
    /// # Errors
    /// Returns an error string if the file cannot be opened or the codec cannot be initialized.
    pub fn new(filename: &str) -> Result<Self, String> {
        if !Path::new(filename).exists() {
            return Err(format!("File not found: {filename}"));
        }

        let filename_c = CString::new(filename).map_err(|_| "Invalid filename")?;

        let mut format_context: *mut raw::AVFormatContext = ptr::null_mut();
        let ret = unsafe { raw::avformat_open_input(&raw mut format_context, filename_c.as_ptr(), ptr::null(), ptr::null_mut()) };
        if ret < 0 {
            return Err(format!("avformat_open_input failed: {}", get_error_string(ret)));
        }

        let ret = unsafe { raw::avformat_find_stream_info(format_context, ptr::null_mut()) };
        if ret < 0 {
            unsafe { raw::avformat_close_input(&raw mut format_context) };
            return Err(format!("avformat_find_stream_info failed: {}", get_error_string(ret)));
        }

        let video_stream = unsafe { raw::av_find_best_stream(format_context, raw::AVMediaType_AVMEDIA_TYPE_VIDEO, -1, -1, ptr::null_mut(), 0) };
        if video_stream < 0 {
            unsafe { raw::avformat_close_input(&raw mut format_context) };
            return Err("No video stream found".to_string());
        }

        let stream_index = video_stream as c_int;
        let stream = unsafe { *(*format_context).streams.add(stream_index as usize) };

        let width = unsafe { (*stream).codecpar.as_ref().map_or(0, |p| p.width) };
        let height = unsafe { (*stream).codecpar.as_ref().map_or(0, |p| p.height) };
        let codec_id = unsafe { (*stream).codecpar.as_ref().map_or(0, |p| p.codec_id) };

        let codec = unsafe { raw::avcodec_find_decoder(codec_id) };
        if codec.is_null() {
            unsafe { raw::avformat_close_input(&raw mut format_context) };
            return Err("Codec not found".to_string());
        }

        let mut codec_context = unsafe { raw::avcodec_alloc_context3(codec) };
        if codec_context.is_null() {
            unsafe { raw::avformat_close_input(&raw mut format_context) };
            return Err("Could not allocate codec context".to_string());
        }

        let ret = unsafe { raw::avcodec_parameters_to_context(codec_context, (*stream).codecpar) };
        if ret < 0 {
            unsafe { raw::avcodec_free_context(&raw mut codec_context) };
            unsafe { raw::avformat_close_input(&raw mut format_context) };
            return Err(format!("avcodec_parameters_to_context failed: {}", get_error_string(ret)));
        }

        let ret = unsafe { raw::avcodec_open2(codec_context, codec, ptr::null_mut()) };
        if ret < 0 {
            unsafe { raw::avcodec_free_context(&raw mut codec_context) };
            unsafe { raw::avformat_close_input(&raw mut format_context) };
            return Err(format!("avcodec_open2 failed: {}", get_error_string(ret)));
        }

        let fr = unsafe { (*stream).r_frame_rate };
        let frame_rate = if fr.den > 0 { f64::from(fr.num) / f64::from(fr.den) } else { 30.0 };

        let sws_context = unsafe {
            raw::sws_getContext(
                width, height, raw::AVPixelFormat_AV_PIX_FMT_YUV420P as c_int,
                width, height, raw::AVPixelFormat_AV_PIX_FMT_RGBA as c_int,
                1, ptr::null_mut(), ptr::null_mut(), ptr::null(),
            )
        };

        let mut frame = unsafe { raw::av_frame_alloc() };
        let mut rgba_frame = unsafe { raw::av_frame_alloc() };
        let mut packet = unsafe { raw::av_packet_alloc() };
        let mut rgba_data = vec![0u8; (width * height * 4) as usize];

        if rgba_frame.is_null() || frame.is_null() || packet.is_null() {
            if !rgba_frame.is_null() { unsafe { raw::av_frame_free(&raw mut rgba_frame) }; }
            if !frame.is_null() { unsafe { raw::av_frame_free(&raw mut frame) }; }
            if !packet.is_null() { unsafe { raw::av_packet_free(&raw mut packet) }; }
            if !sws_context.is_null() { unsafe { raw::sws_freeContext(sws_context) }; }
            unsafe { raw::avcodec_free_context(&raw mut codec_context) };
            unsafe { raw::avformat_close_input(&raw mut format_context) };
            return Err("Could not allocate frame or packet".to_string());
        }

        unsafe {
            (*rgba_frame).width = width;
            (*rgba_frame).height = height;
            (*rgba_frame).format = raw::AVPixelFormat_AV_PIX_FMT_RGBA as c_int;
            (*rgba_frame).data[0] = rgba_data.as_mut_ptr();
        }

        Ok(Self {
            format_context,
            codec_context,
            sws_context,
            video_stream_index: stream_index,
            width,
            height,
            frame_rate,
            current_frame: 0,
            frame,
            rgba_frame,
            packet,
            rgba_data,
            eof: false,
        })
    }

    #[must_use]
    pub const fn width(&self) -> i32 { self.width }
    #[must_use]
    pub const fn height(&self) -> i32 { self.height }
    #[must_use]
    pub const fn frame_rate(&self) -> f64 { self.frame_rate }
    #[must_use]
    pub fn rgba_data(&self) -> &[u8] { &self.rgba_data }
    #[must_use]
    pub const fn current_frame(&self) -> i32 { self.current_frame }

    /// # Errors
    /// Returns an error string if decoding fails. Returns `Ok(false)` at end of stream.
    pub fn decode_frame(&mut self) -> Result<bool, String> {
        if self.eof {
            return Ok(false);
        }

        loop {
            let ret = unsafe { raw::avcodec_receive_frame(self.codec_context, self.frame) };
            if ret == 0 {
                let frame = unsafe { &*self.frame };
                let src_data: [*const u8; 8] = [
                    frame.data[0], frame.data[1], frame.data[2], frame.data[3],
                    frame.data[4], frame.data[5], frame.data[6], frame.data[7],
                ];
                let src_stride: [c_int; 8] = [
                    frame.linesize[0], frame.linesize[1], frame.linesize[2], frame.linesize[3],
                    frame.linesize[4], frame.linesize[5], frame.linesize[6], frame.linesize[7],
                ];

                let dst_data: [*mut u8; 8] = [
                    self.rgba_data.as_mut_ptr(), ptr::null_mut(), ptr::null_mut(), ptr::null_mut(),
                    ptr::null_mut(), ptr::null_mut(), ptr::null_mut(), ptr::null_mut(),
                ];
                let dst_stride: [c_int; 8] = [self.width * 4, 0, 0, 0, 0, 0, 0, 0];

                unsafe {
                    raw::sws_scale(
                        self.sws_context,
                        src_data.as_ptr(),
                        src_stride.as_ptr(),
                        0,
                        self.height,
                        dst_data.as_ptr(),
                        dst_stride.as_ptr(),
                    );
                }
                return Ok(true);
            } else if ret == -libc::EAGAIN {
                let ret = unsafe { raw::av_read_frame(self.format_context, self.packet) };
                if ret < 0 {
                    if ret == AVERROR_EOF {
                        self.eof = true;
                        unsafe { raw::avcodec_send_packet(self.codec_context, ptr::null()) };
                        continue;
                    }
                    return Err(format!("av_read_frame failed: {}", get_error_string(ret)));
                } else if unsafe { (*self.packet).stream_index } != self.video_stream_index {
                    unsafe { raw::av_packet_unref(self.packet) };
                    continue;
                }
                let ret = unsafe { raw::avcodec_send_packet(self.codec_context, self.packet) };
                unsafe { raw::av_packet_unref(self.packet) };
                if ret < 0 {
                    if ret == -libc::EAGAIN {
                        continue;
                    }
                    return Err(format!("avcodec_send_packet failed: {}", get_error_string(ret)));
                }
            } else {
                return Err(format!("avcodec_receive_frame failed: {}", get_error_string(ret)));
            }
        }
    }

    /// # Errors
    /// Returns an error string if seeking fails.
    pub fn seek(&mut self, timestamp: i64) -> Result<(), String> {
        let ret = unsafe { raw::av_seek_frame(self.format_context, self.video_stream_index, timestamp, 0) };
        if ret < 0 {
            return Err(format!("av_seek_frame failed: {}", get_error_string(ret)));
        }
        unsafe { raw::avcodec_flush_buffers(self.codec_context) };
        self.eof = false;
        Ok(())
    }

    pub const fn reset(&mut self) {
        self.eof = false;
        self.current_frame = 0;
    }

    pub const fn set_current_frame(&mut self, frame: i32) {
        self.current_frame = frame;
    }

    fn cleanup(&mut self) {
        if !self.frame.is_null() {
            unsafe { raw::av_frame_free(&raw mut self.frame) };
            self.frame = ptr::null_mut();
        }
        if !self.rgba_frame.is_null() {
            unsafe { raw::av_frame_free(&raw mut self.rgba_frame) };
            self.rgba_frame = ptr::null_mut();
        }
        if !self.packet.is_null() {
            unsafe { raw::av_packet_free(&raw mut self.packet) };
            self.packet = ptr::null_mut();
        }
        if !self.sws_context.is_null() {
            unsafe { raw::sws_freeContext(self.sws_context) };
            self.sws_context = ptr::null_mut();
        }
        if !self.codec_context.is_null() {
            unsafe { raw::avcodec_free_context(&raw mut self.codec_context) };
            self.codec_context = ptr::null_mut();
        }
        if !self.format_context.is_null() {
            unsafe { raw::avformat_close_input(&raw mut self.format_context) };
            self.format_context = ptr::null_mut();
        }
        self.rgba_data.clear();
    }
}

impl Drop for VideoDecoder {
    fn drop(&mut self) {
        self.cleanup();
    }
}

fn get_error_string(errnum: c_int) -> String {
    unsafe {
        let mut errbuf = [0u8; 256];
        raw::av_strerror(errnum, errbuf.as_mut_ptr().cast::<std::os::raw::c_char>(), 256);
        CStr::from_ptr(errbuf.as_ptr().cast::<std::os::raw::c_char>())
            .to_string_lossy()
            .into_owned()
    }
}
