import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';

// Opt-in GNOME Shell extension: exposes app identifier only, not window title.
const XML = `<node>
 <interface name="org.workforceintelligence.Focus1">
  <method name="GetFocus">
   <arg type="s" name="app_id" direction="out"/>
   <arg type="x" name="at_ms" direction="out"/>
  </method>
  <signal name="FocusChanged">
   <arg name="app_id" type="s"/>
   <arg name="at_ms" type="x"/>
  </signal>
 </interface>
</node>`;
const BUS_NAME = 'org.workforceintelligence.Focus';
const PATH = '/org/workforceintelligence/Focus';

export default class WorkforceFocus extends Extension {
  enable() {
    this._connection = Gio.DBus.session;
    this._exported = Gio.DBusExportedObject.wrapJSObject(XML, this);
    this._exported.export(this._connection, PATH);
    this._nameId = Gio.bus_own_name(
      Gio.BusType.SESSION, BUS_NAME, Gio.BusNameOwnerFlags.NONE,
      null, null, null
    );
    this._focusSignal = global.display.connect('notify::focus-window', () => this._emitFocus());
    this._emitFocus();
  }

  _appId() {
    const window = global.display.get_focus_window();
    if (!window) return '';
    try {
      return window.get_gtk_application_id?.() ||
        window.get_sandboxed_app_id?.() ||
        window.get_wm_class?.() || '';
    } catch (_) {
      return '';
    }
  }

  GetFocus() {
    return [this._appId(), Math.trunc(GLib.get_real_time() / 1000)];
  }

  _emitFocus() {
    if (!this._exported) return;
    const millis = Math.trunc(GLib.get_real_time() / 1000);
    this._exported.emit_signal('FocusChanged', new GLib.Variant('(sx)', [this._appId(), millis]));
  }

  disable() {
    if (this._focusSignal) global.display.disconnect(this._focusSignal);
    if (this._nameId) Gio.bus_unown_name(this._nameId);
    this._exported?.unexport();
    this._focusSignal = null;
    this._nameId = null;
    this._exported = null;
    this._connection = null;
  }
}
