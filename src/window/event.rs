//! @author 十四叔
//! @date 2026/07/17

//! 应用 → Handler 的事件通道 + winit 事件 → 内部事件的适配。
//!
//! `WindowAppEvent` 是 App 主动发给窗口的事件 (显隐 / 退出 / 阶段流转通知),
//! 通过 mpsc 通道 (`WindowEventSender`) 发到主线程，Handler 在 `about_to_wait` 轮询。
//!
//! `convert_event` 是 winit 原生事件 → 丹青平台无关事件的适配器，无相关事件返 None。

use std::sync::mpsc::Sender;

use winit::{
    event::{ElementState, Ime as WinitIme, MouseButton as WinitMouseButton, WindowEvent},
    keyboard::{Key as WinitKey, ModifiersState, NamedKey as WinitNamedKey},
};

use crate::Color;
use crate::Point;
use crate::event::{Event, ImeEvent, Key, MouseButton, NamedKey};

/// 应用主动发给窗口的事件 (用于全局热键配套：显隐 / 退出等)。
#[derive(Debug, Clone, Copy)]
pub enum WindowAppEvent {
    /// 切换窗口可见性 (Handler 翻转内部状态后应用到 winit)。
    /// 单一事实源在 Handler, App 不持有副本以避免失同步。
    ToggleVisible,
    /// 仅显示窗口 (不切换)。用于 focus_lost 等场景：窗口已隐藏时不再重复显示。
    ShowWindow,
    /// 仅隐藏窗口 (不切换)。用于 focus_lost 和关闭按钮：避免 toggle 导致的反复显隐。
    HideWindow,
    /// 退出应用 (事件循环收到后 `event_loop.exit()`)。
    Quit,
    /// 阶段流转通知：隐藏态时 Handler 自动呼出窗口 + 抢焦点。
    PhaseAdvanced,
    /// 动态更新窗口背景色 (主题切换等场景)。
    SetClearColor(Color),
}

/// 应用持有的窗口事件发送器 (轻量 clone, 内部是 mpsc Sender)。
#[derive(Clone)]
pub struct WindowEventSender {
    pub(super) sender: Sender<WindowAppEvent>,
}

impl WindowEventSender {
    /// 请求 Handler 翻转窗口可见性。
    pub fn toggle_visible(&self) {
        let _ = self.sender.send(WindowAppEvent::ToggleVisible);
    }

    /// 请求 Handler 显示窗口 (仅显示，不切换)。
    /// 用于 focus_lost 等场景：窗口已隐藏时不再重复显示。
    pub fn show_window(&self) {
        let _ = self.sender.send(WindowAppEvent::ShowWindow);
    }

    /// 请求 Handler 隐藏窗口 (仅隐藏，不切换)。
    /// 用于 focus_lost 和关闭按钮：避免 toggle 导致的反复显隐。
    pub fn hide_window(&self) {
        let _ = self.sender.send(WindowAppEvent::HideWindow);
    }

    /// 退出应用。
    pub fn quit(&self) {
        let _ = self.sender.send(WindowAppEvent::Quit);
    }

    /// 通知 Handler 阶段已流转 (隐藏态时 Handler 决定是否自动呼出)。
    pub fn phase_advanced(&self) {
        let _ = self.sender.send(WindowAppEvent::PhaseAdvanced);
    }

    /// 动态更新窗口背景色。
    pub fn set_clear_color(&self, color: Color) {
        let _ = self.sender.send(WindowAppEvent::SetClearColor(color));
    }
}

/// 把 winit 窗口事件转换为内部事件; 无关事件返回 None。
pub(super) fn convert_event(
    event: &WindowEvent,
    cursor: Point,
    modifiers: ModifiersState,
) -> Option<Event> {
    match event {
        WindowEvent::CursorMoved { position, .. } => Some(Event::CursorMoved(Point::new(
            position.x as f32,
            position.y as f32,
        ))),
        WindowEvent::CursorLeft { .. } => Some(Event::CursorLeft),
        WindowEvent::MouseInput { state, button, .. } => {
            let button = match button {
                WinitMouseButton::Left => MouseButton::Left,
                WinitMouseButton::Right => MouseButton::Right,
                WinitMouseButton::Middle => MouseButton::Middle,
                WinitMouseButton::Back => MouseButton::Back,
                WinitMouseButton::Forward => MouseButton::Forward,
                WinitMouseButton::Other(v) => MouseButton::Other(*v),
            };
            Some(Event::MouseInput {
                button,
                pressed: *state == ElementState::Pressed,
                position: cursor,
            })
        }
        WindowEvent::MouseWheel { delta, .. } => {
            let d = match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                winit::event::MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
            };
            Some(Event::MouseWheel {
                delta: d,
                position: cursor,
            })
        }
        WindowEvent::KeyboardInput { event, .. } => {
            let key = match &event.logical_key {
                WinitKey::Character(s) => Key::Character(s.to_string()),
                WinitKey::Named(named) => {
                    let named = match named {
                        WinitNamedKey::ArrowUp => NamedKey::ArrowUp,
                        WinitNamedKey::ArrowDown => NamedKey::ArrowDown,
                        WinitNamedKey::ArrowLeft => NamedKey::ArrowLeft,
                        WinitNamedKey::ArrowRight => NamedKey::ArrowRight,
                        WinitNamedKey::Space => NamedKey::Space,
                        WinitNamedKey::Enter => NamedKey::Enter,
                        WinitNamedKey::Escape => NamedKey::Escape,
                        WinitNamedKey::Tab => NamedKey::Tab,
                        WinitNamedKey::Backspace => NamedKey::Backspace,
                        WinitNamedKey::Delete => NamedKey::Delete,
                        WinitNamedKey::Home => NamedKey::Home,
                        WinitNamedKey::End => NamedKey::End,
                        WinitNamedKey::Shift => NamedKey::Shift,
                        WinitNamedKey::Control => NamedKey::Control,
                        WinitNamedKey::Alt => NamedKey::Alt,
                        _ => return None,
                    };
                    Key::Named(named)
                }
                _ => return None,
            };
            Some(Event::Key {
                key,
                pressed: event.state == ElementState::Pressed,
                shift: modifiers.shift_key(),
                ctrl: modifiers.control_key(),
                alt: modifiers.alt_key(),
            })
        }
        WindowEvent::Ime(ime) => match ime {
            WinitIme::Enabled => Some(Event::Ime(ImeEvent::Enabled)),
            WinitIme::Disabled => Some(Event::Ime(ImeEvent::Disabled)),
            WinitIme::Preedit(value, cursor) => Some(Event::Ime(ImeEvent::Preedit {
                value: value.clone(),
                cursor: *cursor,
            })),
            WinitIme::Commit(value) => Some(Event::Ime(ImeEvent::Commit {
                value: value.clone(),
            })),
        },
        _ => None,
    }
}
