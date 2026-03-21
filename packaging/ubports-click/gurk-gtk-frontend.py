#!/usr/bin/env python3
import os
import signal
import sys

try:
    import gi
except ImportError as exc:
    raise SystemExit(f"failed to import PyGObject: {exc}")

gi.require_version("Gtk", "3.0")
gi.require_version("Vte", "2.91")

from gi.repository import GLib, Gtk, Pango, Vte


class GurkFrontend(Gtk.Application):
    def __init__(self, backend_argv):
        super().__init__(application_id="org.boxdot.gurk")
        self._backend_argv = backend_argv
        self._exit_status = 1
        self._terminal = None
        self._child_pid = None
        signal.signal(signal.SIGINT, signal.SIG_DFL)

    def do_activate(self):
        if self.props.active_window is not None:
            self.props.active_window.present()
            return

        window = Gtk.ApplicationWindow(application=self, title="Gurk")
        window.set_default_size(960, 640)
        window.set_icon_name("utilities-terminal")
        window.connect("delete-event", self._on_delete_event)

        terminal = Vte.Terminal()
        terminal.set_font(Pango.FontDescription("Monospace 11"))
        terminal.set_scrollback_lines(0)
        terminal.set_rewrap_on_resize(False)
        terminal.set_mouse_autohide(False)
        terminal.set_allow_hyperlink(True)
        terminal.connect("child-exited", self._on_child_exited)
        terminal.connect("notify::window-title", self._on_title_changed, window)

        scroller = Gtk.ScrolledWindow()
        scroller.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        scroller.add(terminal)

        window.add(scroller)
        window.show_all()

        self._terminal = terminal
        self._spawn_backend()

    def _spawn_backend(self):
        env = os.environ.copy()
        env["TERM"] = env.get("TERM", "xterm-256color")
        envv = [f"{key}={value}" for key, value in env.items()]

        self._terminal.spawn_async(
            Vte.PtyFlags.DEFAULT,
            None,
            self._backend_argv,
            envv,
            GLib.SpawnFlags(0),
            None,
            None,
            -1,
            None,
            self._on_spawn_complete,
            None,
        )

    def _on_spawn_complete(self, _terminal, pid, error, _user_data):
        if error is not None:
            self._exit_status = 1
            self._show_error(f"failed to start gurk: {error.message}")
            self.quit()
            return

        self._child_pid = pid

    def _on_child_exited(self, _terminal, status):
        if os.WIFEXITED(status):
            self._exit_status = os.WEXITSTATUS(status)
        elif os.WIFSIGNALED(status):
            self._exit_status = 128 + os.WTERMSIG(status)
        else:
            self._exit_status = 1
        self.quit()

    def _on_delete_event(self, _window, _event):
        if self._child_pid is not None:
            try:
                os.kill(self._child_pid, signal.SIGHUP)
            except ProcessLookupError:
                pass
        return False

    def _on_title_changed(self, terminal, _pspec, window):
        title = terminal.get_window_title() or "Gurk"
        window.set_title(title)

    def _show_error(self, message):
        window = self.props.active_window
        dialog = Gtk.MessageDialog(
            transient_for=window,
            flags=Gtk.DialogFlags.MODAL,
            message_type=Gtk.MessageType.ERROR,
            buttons=Gtk.ButtonsType.CLOSE,
            text="Gurk failed to start",
        )
        dialog.format_secondary_text(message)
        dialog.run()
        dialog.destroy()


def main():
    if len(sys.argv) < 2:
        raise SystemExit("usage: gurk-gtk-frontend.py /path/to/gurk [args...]")

    backend = sys.argv[1:]
    app = GurkFrontend(backend)
    app.run(sys.argv[:1])
    raise SystemExit(app._exit_status)


if __name__ == "__main__":
    main()
