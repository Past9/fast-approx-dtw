# fast-approx-dtw

This is an experimental implementation of the [Dynamic Time Warping](https://en.wikipedia.org/wiki/Dynamic_time_warping) algorithm that focuses on performance and usability in embedded environments. It's a `#[no_std]` crate that runs entirely in the stack.

Basic DTW implementations have quadratic time complexity due to the need to calculate errors and paths over an NxN (where N is the signal length) grid. This implementation* repeatedly downsamples the signals to half their size until they can't be evenly divided by 2 anymore. It then solves the smallest downsample and uses the generated path to guide the solution of the next largest one, only calculating errors and paths that lie near the downsampled path. It works its way back up the "stack" of downsamples until it solves the original input signals, resulting in linear time complexity.

There are situations where a downsampled signal can generate a path that's wildly different from the most correct path on the upsampled signal. In that case, this implementation will not generate the most optimal path, hence the "approximate" in `fast-approx-dtw`. This appears to only be an issue if the input signals are vastly different from each other. This library's main goal is to eventually be useful for generating depth maps between stereo images in real time on embedded devices. Since these images should be very similar to each other, this isn't expected to be an issue. 

\* Based on [FastDTW: Toward Accurate Dynamic Time Warping in Linear Time and Space](https://www.semanticscholar.org/paper/FastDTW%3A-Toward-Accurate-Dynamic-Time-Warping-in-Salvador-Chan/05a20cde15e172fc82f32774dd0cf4fe5827cad2)
