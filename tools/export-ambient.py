#!/usr/bin/env python3
"""Export danqing pomodoro ambient sound beds for procedurally added scenes.

Generates procedural audio for the4 new scenes (铁匠铺/洞穴/夜市/火车):
filtered noise (FFT-shaped) + slow gust envelope + distinctive tonal elements.
Deterministic (fixed seeds), loop-safe (stationary noise + seam crossfade),
zero external assets.

Outputs:
    assets/audio/blacksmith.ogg
    assets/audio/cave.ogg
    assets/audio/nightmarket.ogg
    assets/audio/train.ogg

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
    if kind == "blacksmith":
        # 铁匠铺: 锻造环境噪音 (中低频为主, 火焰低鸣)
        body = 0.7 / (1.0 + ((freqs - 300.0) / 200.0) ** 2.0)  # 锻造环境
        fire = 0.4 / (1.0 + ((freqs - 150.0) / 100.0) ** 2.0)  # 火焰低鸣
        metal = 0.3 / (1.0 + ((freqs - 2000.0) / 500.0) ** 2.0)  # 金属共鸣
        hiss = 0.8 / (1.0 + (freqs / 4000.0) ** 6.0)
        return np.clip(body * 0.5 + fire * 0.3 + metal * 0.15, 0.0, 1.0) * hiss
    elif kind == "cave":
        # 洞穴: 潮湿环境 + 深处共鸣
        body = 0.5 / (1.0 + ((freqs - 200.0) / 150.0) ** 2.0)  # 潮湿环境
        drip = 0.8 / (1.0 + ((freqs - 1500.0) / 300.0) ** 2.0)  # 滴水频率
        echo = 0.6 / (1.0 + ((freqs - 800.0) / 200.0) ** 2.0)  # 深处共鸣
        hiss = 0.7 / (1.0 + (freqs / 3000.0) ** 6.0)
        return np.clip(body * 0.4 + drip * 0.25 + echo * 0.2, 0.0, 1.0) * hiss
    elif kind == "nightmarket":
        # 夜市: 人声嘈杂 + 食物蒸汽 + 灯笼嗡鸣
        body = 0.6 / (1.0 + ((freqs - 500.0) / 300.0) ** 2.0)  # 人声基底
        chatter = 0.8 / (1.0 + ((freqs - 1000.0) / 400.0) ** 2.0)  # 人声嘈杂
        steam = 0.4 / (1.0 + ((freqs - 2500.0) / 600.0) ** 2.0)  # 蒸汽嘶嘶
        hiss = 0.9 / (1.0 + (freqs / 5000.0) ** 6.0)
        return np.clip(body * 0.4 + chatter * 0.3 + steam * 0.15, 0.0, 1.0) * hiss
    elif kind == "train":
        # 火车: 车厢轰鸣 + 铁轨节奏 + 风声
        body = 0.7 / (1.0 + ((freqs - 200.0) / 150.0) ** 2.0)  # 车厢低频轰鸣
        track = 0.5 / (1.0 + ((freqs - 400.0) / 200.0) ** 2.0)  # 铁轨振动
        wind = 0.3 / (1.0 + ((freqs - 1500.0) / 500.0) ** 2.0)  # 车窗风声
        hiss = 0.8 / (1.0 + (freqs / 4000.0) ** 6.0)
        return np.clip(body * 0.5 + track * 0.3 + wind * 0.15, 0.0, 1.0) * hiss
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


def _tonal_elements(kind: str, seed: int) -> np.ndarray:
    """Generate distinctive tonal elements per scene (stereo)."""
    rng = np.random.default_rng(seed)

    if kind == "blacksmith":
        # 铁匠铺: 真实锤击声 (噪声瞬态 + 金属共振模式)
        n_strikes = 32
        onsets = rng.uniform(0, DUR, n_strikes)
        signal = np.zeros(N)
        for onset in onsets:
            start = int(onset * SR)
            dur = rng.uniform(0.1, 0.2)
            length = int(dur * SR)
            if start + length >= N:
                continue
            t = np.linspace(0, dur, length)
            # 瞬态: 噪声 burst (锤击瞬间)
            transient_len = int(0.005 * SR)
            transient = rng.uniform(-1, 1, min(transient_len, length))
            # 金属共振: 多个非谐波模式
            modes = [800, 1100, 2300, 3500, 4800]  # 非谐波频率
            resonance = np.zeros(length)
            for mode_f in modes:
                mode_amp = rng.uniform(0.1, 0.3)
                mode_decay = rng.uniform(15, 30)
                resonance += mode_amp * np.sin(2 * np.pi * mode_f * t) * np.exp(-t * mode_decay)
            # 组合: 瞬态 + 共振
            tone = np.zeros(length)
            tone[:len(transient)] += transient * 0.6
            tone += resonance * 0.4
            tone *= np.exp(-t * 20.0)  # 整体衰减
            signal[start:start + length] += tone
        # 风箱: 噪声调制低频
        for _ in range(8):
            onset = rng.uniform(0, DUR)
            start = int(onset * SR)
            length = int(rng.uniform(1.0, 2.0) * SR)
            if start + length >= N:
                continue
            t = np.linspace(0, length / SR, length)
            # 呼吸: 低频正弦 + 噪声
            breath = np.sin(2 * np.pi * 0.3 * t) * 0.2
            breath += rng.uniform(-0.08, 0.08, length) * np.sin(2 * np.pi * 0.2 * t)
            signal[start:start + length] += breath
        pan = rng.uniform(0.3, 0.7, n_strikes)
        left = signal.copy()
        right = signal.copy()
        for i, onset in enumerate(onsets):
            start = int(onset * SR)
            length = int(0.2 * SR)
            if start + length >= N:
                continue
            left[start:start + length] *= pan[i]
            right[start:start + length] *= (1.0 - pan[i])
        return np.stack([left, right], axis=1)

    elif kind == "cave":
        # 洞穴: 真实水滴声 (频率扫描 + 洞穴混响)
        n_drops = 40
        onsets = rng.uniform(0, DUR, n_drops)
        signal = np.zeros(N)
        for onset in onsets:
            start = int(onset * SR)
            dur = 0.15
            length = int(dur * SR)
            if start + length >= N:
                continue
            t = np.linspace(0, dur, length)
            # 水滴: 频率快速下降 (bloop声)
            f_start = rng.uniform(2500, 4500)
            f_end = rng.uniform(400, 800)
            # 指数下降
            freq = f_start * np.exp(-t * 8.0) + f_end
            phase = 2 * np.pi * np.cumsum(freq) / SR
            # 包络: 快速attack + 指数decay
            env = np.exp(-t * 15.0)
            env *= np.minimum(t / 0.001, 1.0)  # 1ms attack
            tone = np.sin(phase) * env * 0.5
            signal[start:start + length] += tone
            # 洞穴混响: 多次反射 (模拟大空间)
            for delay, gain, damp in [(0.08, 0.6, 0.7), (0.18, 0.4, 0.5), (0.32, 0.25, 0.3), (0.5, 0.15, 0.2)]:
                echo_start = start + int(delay * SR)
                echo_len = int(0.4 * SR)
                if echo_start + echo_len < N:
                    echo_t = np.linspace(0, 0.4, echo_len)
                    echo_freq = f_start * np.exp(-echo_t * 8.0) + f_end
                    echo_phase = 2 * np.pi * np.cumsum(echo_freq) / SR
                    echo_env = np.exp(-echo_t * 5.0) * gain
                    echo_tone = np.sin(echo_phase) * echo_env * 0.3
                    signal[echo_start:echo_start + echo_len] += echo_tone
        return np.stack([signal, signal * 0.9], axis=1)

    elif kind == "nightmarket":
        # 夜市: 真实人群嘈杂 (多声源叠加 + 背景babble)
        signal = np.zeros(N)
        # 背景babble: 多人说话的混合噪声
        babble_len = N
        babble = np.zeros(babble_len)
        for _ in range(30):  # 30个声源
            f0 = rng.uniform(80, 250)  # 基频范围
            onset = rng.uniform(0, DUR)
            duration = rng.uniform(1.0, 3.0)
            start = int(onset * SR)
            length = int(duration * SR)
            if start + length >= N:
                continue
            t = np.linspace(0, duration, length)
            # 声带: 脉冲序列
            vocal = np.zeros(length)
            period_samples = int(SR / f0)
            for p in range(0, length, period_samples):
                pulse_len = min(int(0.003 * SR), length - p)
                vocal[p:p+pulse_len] = rng.uniform(-1, 1, pulse_len)
            # 共振峰滤波 (简化: 用多个正弦波模拟)
            f1 = rng.uniform(300, 800)
            f2 = rng.uniform(800, 1500)
            formants = np.sin(2 * np.pi * f1 * t) * 0.4 + np.sin(2 * np.pi * f2 * t) * 0.2
            voice = vocal * formants
            # 语调
            voice *= (0.3 + 0.7 * np.abs(np.sin(2 * np.pi * rng.uniform(0.1, 0.3) * t)))
            # 淡入淡出
            fade = min(int(0.1 * SR), length // 4)
            if fade > 0:
                voice[:fade] *= np.linspace(0, 1, fade)
                voice[-fade:] *= np.linspace(1, 0, fade)
            signal[start:start + length] += voice * rng.uniform(0.1, 0.25)
        # 翻炒声: 噪声 + 金属共振
        n_fries = 20
        for _ in range(n_fries):
            onset = rng.uniform(0, DUR)
            start = int(onset * SR)
            length = int(rng.uniform(0.05, 0.15) * SR)
            if start + length >= N:
                continue
            t = np.linspace(0, length / SR, length)
            # 金属摩擦: 噪声 * 高频正弦
            metal = rng.uniform(-1, 1, length) * np.sin(2 * np.pi * rng.uniform(4000, 7000) * t)
            metal *= np.exp(-t * 25.0) * 0.3
            signal[start:start + length] += metal
        return np.stack([signal, signal * 0.85], axis=1)

    elif kind == "train":
        # 火车: 真实铁轨声 (节奏性噪声 + 低频轰鸣)
        bpm = rng.uniform(120, 160)
        beat_interval = 60.0 / bpm
        n_beats = int(DUR / beat_interval)
        signal = np.zeros(N)
        for i in range(n_beats):
            onset = i * beat_interval
            start = int(onset * SR)
            # 双击 (咔-嗒) - 模拟车轮经过铁轨接缝
            for offset, strength in [(0.0, 1.0), (0.15, 0.7)]:
                click_start = start + int(offset * SR)
                length = int(0.04 * SR)
                if click_start + length >= N:
                    continue
                t = np.linspace(0, 0.04, length)
                # 金属撞击: 噪声 * 带通
                noise = rng.uniform(-1, 1, length)
                # 带通: 2000-4000 Hz (金属感)
                band = np.sin(2 * np.pi * rng.uniform(2000, 4000) * t)
                click = noise * band * strength
                click *= np.exp(-t * 80.0) * 0.5
                # 低频冲击
                click += np.sin(2 * np.pi * 120 * t) * np.exp(-t * 40.0) * 0.2
                signal[click_start:click_start + length] += click
        # 车厢: 持续低频轰鸣 + 风声
        for _ in range(6):
            onset = rng.uniform(0, DUR)
            start = int(onset * SR)
            length = int(rng.uniform(2.0, 4.0) * SR)
            if start + length >= N:
                continue
            t = np.linspace(0, length / SR, length)
            # 轰鸣: 低频噪声 (不是正弦)
            rumble = rng.uniform(-1, 1, length)
            # 低通滤波 (简化: 用低频正弦调制)
            rumble *= np.sin(2 * np.pi * rng.uniform(50, 100) * t)
            rumble *= np.hanning(length) * 0.15
            signal[start:start + length] += rumble
        return np.stack([signal, signal * 0.9], axis=1)

    return np.zeros((N, 2))


def _render(kind: str, seed: int, rms_target: float, depth: float = 0.55) -> np.ndarray:
    """Stereo bed: noise + tonal elements, gust envelope, normalized."""
    # 噪声基底 (降低为背景纹理, 不盖住音色元素)
    left = _colored_noise(kind, seed) * _gust_envelope(seed + 1, period=9.0, depth=depth) * 0.15
    right = _colored_noise(kind, seed + 7) * _gust_envelope(seed + 9, period=12.0, depth=depth) * 0.15
    # 音色元素 (保持原音量, 作为主体)
    tonal = _tonal_elements(kind, seed + 13)
    left += tonal[:, 0]
    right += tonal[:, 1]
    # 归一化
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
    # 铁匠铺: 锻造环境 + 金属锤击
    {"key": "blacksmith", "kind": "blacksmith", "seed": 0xA1B2, "rms_target": 0.06, "depth": 0.45},
    # 洞穴: 潮湿环境 + 滴水回声
    {"key": "cave", "kind": "cave", "seed": 0xC3D4, "rms_target": 0.05, "depth": 0.50},
    # 夜市: 人声嘈杂 + 锅铲翻炒
    {"key": "nightmarket", "kind": "nightmarket", "seed": 0xE5F6, "rms_target": 0.06, "depth": 0.40},
    # 火车: 车厢轰鸣 + 铁轨节拍
    {"key": "train", "kind": "train", "seed": 0x7890, "rms_target": 0.05, "depth": 0.50},
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
