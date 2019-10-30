# vainfo-bis
its like `vainfo` but with the information you actually care about

## How to run
Call vainfo-bis with the device you want to inspect.
```
vainfo-bis /dev/dri/renderD128
# Or via cargo
cargo run /dev/dri/renderD128
```
An example of output from an AMD device might look like
```
libva info: VA-API version 1.2.0
libva info: va_getDriverName() returns 0
libva info: Trying to open /usr/lib64/dri/radeonsi_drv_video.so
libva info: Found init function __vaDriverInit_1_2
libva info: va_openDriver() returns 0
VA-API version 1.2 initialized
VAProfileH264ConstrainedBaseline:
	VAEntrypointEncSlice:
		VAConfigAttribEncMaxRefFrames: P-Frames: 1, B-Frames: 0
		VAConfigAttribEncPackedHeaders:0
		VASurfaceAttribPixelFormat:Ok("NV12")
		VASurfaceAttribMemoryType:0b100000000000000000000000000001
		VASurfaceAttribExternalBufferDescriptor:0
		VASurfaceAttribMaxWidth:4096
		VASurfaceAttribMaxHeight:2304
VAProfileH264Main:
	VAEntrypointEncSlice:
		VAConfigAttribEncMaxRefFrames: P-Frames: 1, B-Frames: 0
		VAConfigAttribEncPackedHeaders:0
		VASurfaceAttribPixelFormat:Ok("NV12")
		VASurfaceAttribMemoryType:0b100000000000000000000000000001
		VASurfaceAttribExternalBufferDescriptor:0
		VASurfaceAttribMaxWidth:4096
		VASurfaceAttribMaxHeight:2304
VAProfileH264High:
	VAEntrypointEncSlice:
		VAConfigAttribEncMaxRefFrames: P-Frames: 1, B-Frames: 0
		VAConfigAttribEncPackedHeaders:0
		VASurfaceAttribPixelFormat:Ok("NV12")
		VASurfaceAttribMemoryType:0b100000000000000000000000000001
		VASurfaceAttribExternalBufferDescriptor:0
		VASurfaceAttribMaxWidth:4096
		VASurfaceAttribMaxHeight:2304
VAProfileHEVCMain:
	VAEntrypointEncSlice:
		VAConfigAttribEncMaxRefFrames: P-Frames: 1, B-Frames: 0
		VAConfigAttribEncPackedHeaders:0
		VASurfaceAttribPixelFormat:Ok("NV12")
		VASurfaceAttribMemoryType:0b100000000000000000000000000001
		VASurfaceAttribExternalBufferDescriptor:0
		VASurfaceAttribMaxWidth:4096
		VASurfaceAttribMaxHeight:2304

```
