#!/usr/bin/env python3
"""Export danqing pomodoro ambient sound beds for the 4 new sky/wind scenes.

The 4 scenes added for the 9-scene immersive world are all "sky/air" realms
(星夜 night breeze, 雪原 snow wind, 沙漠 dry desert wind, 云海 high-altitude
wind). Rather than sourcing external CC0 clips, these are synthesized
procedurally as a wind family: filtered noise (FFT-shaped) + slow gust
envelope. Deterministic (fixed seeds), loop-safe (stationary noise + seam
crossfade), zero external assets — mirrors the "procedural scenes" recipe of
export-scenes.py.

Outputs:
    assets/audio/{starry,snowfield,desert,cloudsea}.ogg   # OGG Vorbis q4

Dependencies:
    pip install numpy        # DSP
    ffmpeg on PATH           # OGG Vorbis encoding

Usage:
    python tools/export-ambient.py
"""

import subprocess
import tempfile
import wave
from pathlib import Path

import numpy as np

REPO_ROOT = Path(__file__).resolve().parent.parent
OUT_DIR = REPO_ROOT / "assets" / "audio"

SR = 44_100
DUR = 48  # seconds (loop-safe by construction)
N = SR * DUR
CROSS = int(SR * 0.3)  # seam crossfade samples


def _spectrum(freqs: np.ndarray, kind: str) -> np.ndarray:
    """Amplitude response 0..1 for the FFT noise shaper, per scene kind."""
    if kind == "night":  # 星夜: deep low rumble + whisper of air
        low = 1.0 / (1.0 + (freqs / 220.0) ** 2.0)
        air = (freqs / 1200.0) ** 1.2 / (1.0 + (freqs / 1200.0) ** 2.0) * 0.35
        return np.clip(low * 0.9 + air, 0.0, 1.0)
    if kind == "desert":  # 沙漠: dry mid-band wind, slight grain
        mid = 1.0 / (1.0 + (freqs / 1300.0) ** 1.4)
        return np.clip(mid, 0.0, 1.0)
    raise ValueError(kind)


def _colored_noise(kind: str, seed: int) -> np.ndarray:
    """Stationary colored noise via FFT shaping (white -> target spectrum)."""
    rng = np.random.default_rng(seed)
    white = rng.standard_normal(N)
    freqs = np.fft.rfftfreq(N, 1.0 / SR)
    amp = _spectrum(freqs, kind)
    amp = amp / amp.max()
    return np.fft.irfft(np.fft.rfft(white) * amp, n=N)


def _gust_envelope(seed: int, period: float = 9.0, depth: float = 0.55) -> np.ndarray:
    """Slow gust envelope (1 - depth .. 1 + depth), loop-safe via seam crossfade."""
    rng = np.random.default_rng(seed)
    steps = int(DUR / period) + 3
    levels = rng.uniform(1.0 - depth, 1.0 + depth, steps)
    t_steps = np.linspace(0, DUR, steps)
    t = np.linspace(0, DUR, N)
    env = np.interp(t, t_steps, levels)
    # Smooth with a 1.2s lowpass (FFT-based; naive convolve is O(N*M) here).
    freqs = np.fft.rfftfreq(N, 1.0 / SR)
    cutoff = 1.0 / (1.0 + (freqs / 0.8) ** 2.0)
    env = np.fft.irfft(np.fft.rfft(env) * cutoff, n=N)
    # Crossfade tail into head so the loop point is seamless.
    if CROSS > 0:
        fade = np.linspace(0.0, 1.0, CROSS)
        head = env[:CROSS]
        env[:CROSS] = head * fade + env[-CROSS:] * (1.0 - fade)
    return np.clip(env, 0.05, None)


def _render(kind: str, seed: int, rms_target: float, depth: float = 0.55) -> np.ndarray:
    """Stereo bed: independent L/R noise, gust envelope, normalized.

    Normalized to a target RMS (not peak): different spectra have different
    crest factors, and peak-normalizing makes narrow-band hiss (snow) much
    louder than broad rumble (starry) for the same peak. Per-scene rms_target
    carries the intended loudness hierarchy. `depth` is the gust-envelope
    modulation (0.12 ≈ steady for waterfall; 0.55 ≈ gusty wind).
    """
    left = _colored_noise(kind, seed) * _gust_envelope(seed + 1, period=9.0, depth=depth)
    right = _colored_noise(kind, seed + 7) * _gust_envelope(seed + 9, period=12.0, depth=depth)
    for ch in (left, right):
        np.tanh(ch, out=ch)
        current = float(np.sqrt(np.mean(ch**2)))
        ch *= rms_target / max(current, 1e-9)
    return np.stack([left, right], axis=1)


def _write_wav(path: Path, stereo: np.ndarray) -> None:
    pcm = (stereo * 32767.0).astype(np.int16)
    with wave.open(str(path), "wb") as wf:
        wf.setnchannels(2)
        wf.setsampwidth(2)
        wf.setframerate(SR)
        wf.writeframes(pcm.tobytes())


def _encode_ogg(wav: Path, out: Path) -> None:
    subprocess.run(
        [
            "ffmpeg", "-y", "-loglevel", "error",
            "-i", str(wav), "-c:a", "libvorbis", "-q:a", "4", str(out),
        ],
        check=True,
    )


SCENES = [
    # 星夜 夜风: 最安静, 低频细语 + 一丝空气感。
    {"key": "starry", "kind": "night", "seed": 0x51A1, "rms_target": 0.05, "depth": 0.55},
    # 沙漠 干风: 干爽中段风声。
    {"key": "desert", "kind": "desert", "seed": 0x53A1, "rms_target": 0.08, "depth": 0.55},
]


def main() -> int:
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    for s in SCENES:
        bed = _render(s["kind"], s["seed"], s["rms_target"], s.get("depth", 0.55))
        with tempfile.TemporaryDirectory() as tmp:
            wav = Path(tmp) / "bed.wav"
            _write_wav(wav, bed)
            out = OUT_DIR / f"{s['key']}.ogg"
            _encode_ogg(wav, out)
        rms = float(np.sqrt(np.mean(bed**2)))
        print(f"[OK ] {s['key']}.ogg  {out.stat().st_size} bytes, RMS {rms:.4f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
