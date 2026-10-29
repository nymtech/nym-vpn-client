/* Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 * SPDX-License-Identifier: GPL-3.0-only
 *
 * NetworkManager editor page for NymVPN. Compiled once against GTK 3 (nm-connection-editor,
 * nm-applet) and once against GTK 4 (GNOME Settings).
 *
 * All NymVPN settings live in the NymVPN desktop app, so the page only says so and opens the
 * app: once when the page is created, and again on every button press. The app is
 * single-instance, so a second launch just raises its window.
 */
#include <gio/gdesktopappinfo.h>
#include <gmodule.h>
#include <gtk/gtk.h>

#include <NetworkManager.h>

#define NYMVPN_DESKTOP_ID "NymVPN.desktop"
#define NYMVPN_COMMAND "nym-vpn-app"
#define NOTICE_TEXT "Settings for this VPN are managed in the NymVPN app."
#define BUTTON_TEXT "Open NymVPN"

#if GTK_CHECK_VERSION(4, 0, 0)
#define box_append(box, child) gtk_box_append(GTK_BOX(box), child)
#define widget_show_all(w) ((void) 0)
#else
#define box_append(box, child) gtk_box_pack_start(GTK_BOX(box), child, FALSE, FALSE, 0)
#define widget_show_all(w) gtk_widget_show_all(w)
#define gtk_label_set_wrap(label, wrap) gtk_label_set_line_wrap(label, wrap)
#endif

#define NYMVPN_TYPE_EDITOR (nymvpn_editor_get_type())
G_DECLARE_FINAL_TYPE(NymvpnEditor, nymvpn_editor, NYMVPN, EDITOR, GObject)

struct _NymvpnEditor {
    GObject parent;
    GtkWidget *widget;
};

static void nymvpn_editor_interface_init(NMVpnEditorInterface *iface);

G_DEFINE_TYPE_EXTENDED(NymvpnEditor, nymvpn_editor, G_TYPE_OBJECT, 0,
                       G_IMPLEMENT_INTERFACE(NM_TYPE_VPN_EDITOR, nymvpn_editor_interface_init))

static void launch_app(void)
{
    GError *error = NULL;
    GAppInfo *info = NULL;
    GDesktopAppInfo *desktop = g_desktop_app_info_new(NYMVPN_DESKTOP_ID);

    if (desktop) {
        info = G_APP_INFO(desktop);
    } else {
        info = g_app_info_create_from_commandline(NYMVPN_COMMAND, "NymVPN",
                                                  G_APP_INFO_CREATE_NONE, &error);
        if (!info) {
            g_warning("NymVPN: cannot build launcher for %s: %s", NYMVPN_COMMAND,
                      error ? error->message : "unknown error");
            g_clear_error(&error);
            return;
        }
    }

    if (!g_app_info_launch(info, NULL, NULL, &error)) {
        g_warning("NymVPN: cannot open the NymVPN app: %s",
                  error ? error->message : "unknown error");
        g_clear_error(&error);
    }
    g_object_unref(info);
}

static void on_open_clicked(GtkButton *button, gpointer user_data)
{
    (void) button;
    (void) user_data;
    launch_app();
}

static GObject *get_widget(NMVpnEditor *editor)
{
    NymvpnEditor *self = NYMVPN_EDITOR(editor);
    return G_OBJECT(self->widget);
}

static gboolean update_connection(NMVpnEditor *editor, NMConnection *connection, GError **error)
{
    (void) editor;
    (void) connection;
    (void) error;
    /* Nothing is editable here; the connection is left exactly as it was. */
    return TRUE;
}

static void nymvpn_editor_dispose(GObject *object)
{
    NymvpnEditor *self = NYMVPN_EDITOR(object);

    g_clear_object(&self->widget);
    G_OBJECT_CLASS(nymvpn_editor_parent_class)->dispose(object);
}

static void nymvpn_editor_init(NymvpnEditor *self)
{
    GtkWidget *box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);
    GtkWidget *label = gtk_label_new(NOTICE_TEXT);
    GtkWidget *button = gtk_button_new_with_label(BUTTON_TEXT);

    gtk_widget_set_margin_top(box, 12);
    gtk_widget_set_margin_bottom(box, 12);
    gtk_widget_set_margin_start(box, 12);
    gtk_widget_set_margin_end(box, 12);
    gtk_label_set_wrap(GTK_LABEL(label), TRUE);
    gtk_widget_set_halign(label, GTK_ALIGN_START);
    gtk_widget_set_halign(button, GTK_ALIGN_START);
    g_signal_connect(button, "clicked", G_CALLBACK(on_open_clicked), NULL);

    box_append(box, label);
    box_append(box, button);
    widget_show_all(box);

    self->widget = g_object_ref_sink(box);
}

static void nymvpn_editor_class_init(NymvpnEditorClass *klass)
{
    G_OBJECT_CLASS(klass)->dispose = nymvpn_editor_dispose;
}

static void nymvpn_editor_interface_init(NMVpnEditorInterface *iface)
{
    iface->get_widget = get_widget;
    iface->update_connection = update_connection;
}

__attribute__((visibility("default"))) G_MODULE_EXPORT NMVpnEditor *
nm_vpn_editor_factory_nymvpn(NMVpnEditorPlugin *plugin, NMConnection *connection, GError **error)
{
    (void) plugin;
    (void) connection;
    g_return_val_if_fail(!error || !*error, NULL);

    launch_app();
    return NM_VPN_EDITOR(g_object_new(NYMVPN_TYPE_EDITOR, NULL));
}
