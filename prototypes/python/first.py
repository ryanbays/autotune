from scipy.fft import rfft, rfftfreq, irfft
import time
import soundfile
import numpy as np
import math
import matplotlib.pyplot as plt

output = soundfile.SoundFile("out.wav", "w", samplerate=44100, channels=1)

time_per_window = 0
original = soundfile.SoundFile(input("Enter original filepath: "))
if original.channels > 1:
    raise IOError

current_frame = 0
total_frames = original.frames
t_width = int(input("Enter window size: "))
signal = original.read(t_width)


def create_plot(wave, fourier, result):
    plt.ion()
    fig = plt.figure()
    fig.suptitle(f"Current frames: {current_frame}-{current_frame+t_width}")
    ax1 = fig.add_subplot(2, 2, 1, xlim=[0, t_width], ylim=[-1, 1])
    ax1.set_title("Original wave")
    ax2 = fig.add_subplot(2, 2, 3, ylim=[-10, 10])
    ax2.set_title("Fourier transform")
    ax3 = fig.add_subplot(2, 2, 2, xlim=[0, t_width], ylim=[-1, 1])
    ax3.set_title("Inverse fourier result")

    line1, = ax1.plot(wave, 'r-')  # Returns a tuple of line objects, thus the comma
    line2, = ax2.plot(fourier, 'g-')
    x = []
    for i in range(0, t_width*2):
        x.append(i/2)
    line3, = ax3.plot(x, result, 'b-')
    return fig, line1, line2, line3


def update_plot(fig, lines, wave, fourier, result):
    fig.suptitle(
        f"Current frames: {current_frame}-{current_frame + t_width} ({math.trunc(current_frame / original.samplerate * 100) / 100}s)")
    lines[0].set_ydata(wave)
    lines[1].set_ydata(fourier)
    lines[2].set_ydata(result)
    fig.canvas.draw()
    fig.canvas.flush_events()


def perform_fourier(wave):
    fourier = rfft(wave)

    result = fourier
    reconstruction = irfft(result, t_width*2) * 2
    return fourier, reconstruction


if __name__ == '__main__':
    signal = original.read(t_width)
    f, r = perform_fourier(signal)

    temp = create_plot(signal, f, r)
    fig, lines = temp[0], temp[1:]

    current_frame += t_width
    while True:

        signal = original.read(t_width)
        if not signal.size > 0:
            break
        f, r = perform_fourier(signal)

        update_plot(fig, lines, signal, f, r)

        output.write(r)
        current_frame += t_width

        print(current_frame/total_frames)

    output.close()


