/* Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 * SPDX-License-Identifier: GPL-3.0-only
 *
 * Loader test for the NymVPN NetworkManager editor plugin.
 *
 * Usage: load-plugin <path to libnm-vpn-plugin-nymvpn.so> [<path to nm-nymvpn-service.name>]
 *
 * The core plugin is loaded twice: once without file checks and once through
 * nm_vpn_editor_plugin_load_from_file with the owner and permission checks that
 * nm-connection-editor and GNOME Settings apply (root or current user, no group/other write).
 * When the .name file is given, its [libnm] plugin= entry must be an absolute path.
 *
 * Built three times: without GTK (as nmcli would load the plugin), with GTK 3 and with GTK 4
 * (as nm-connection-editor and GNOME Settings would). The GTK builds also instantiate the editor
 * when a display is available.
 */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include <NetworkManager.h>

#ifdef WITH_GTK
#include <gtk/gtk.h>
#endif

#define SERVICE "org.freedesktop.NetworkManager.nymvpn"

#define CHECK(cond, msg)                                              \
    do {                                                              \
        if (!(cond)) {                                                \
            fprintf(stderr, "FAIL: %s (%s:%d)\n", msg, __FILE__, __LINE__); \
            return 1;                                                 \
        }                                                             \
    } while (0)

int main(int argc, char **argv)
{
    GError *error = NULL;
    NMVpnEditorPlugin *plugin;
    char *name = NULL, *service = NULL;

    if (argc < 2 || argc > 3) {
        fprintf(stderr, "usage: %s <core plugin .so> [<.name file>]\n", argv[0]);
        return 2;
    }

    if (argc == 3) {
        NMVpnPluginInfo *info = nm_vpn_plugin_info_new_from_file(argv[2], &error);
        const char *entry;

        if (!info) {
            fprintf(stderr, "FAIL: .name: %s\n", error ? error->message : "unknown");
            return 1;
        }
        entry = nm_vpn_plugin_info_get_plugin(info);
        CHECK(entry != NULL, ".name has no [libnm] plugin= entry");
        CHECK(g_path_is_absolute(entry), ".name plugin= must be an absolute path");
        CHECK(g_str_has_suffix(entry, "/libnm-vpn-plugin-nymvpn.so"),
              ".name plugin= does not point at libnm-vpn-plugin-nymvpn.so");
        CHECK(g_strcmp0(nm_vpn_plugin_info_get_service(info), SERVICE) == 0,
              ".name service mismatch");
        g_object_unref(info);
    }

    /* The path real hosts take: with owner and permission checks. This must come first, libnm
     * skips the checks for a module that is already loaded in the process. */
    plugin = nm_vpn_editor_plugin_load_from_file(argv[1], SERVICE, getuid(), NULL, NULL, &error);
    if (!plugin) {
        fprintf(stderr, "FAIL: load_from_file (checked): %s\n", error ? error->message : "unknown");
        return 1;
    }
    g_object_unref(plugin);

    plugin = nm_vpn_editor_plugin_load(argv[1], SERVICE, &error);
    if (!plugin) {
        fprintf(stderr, "FAIL: load: %s\n", error ? error->message : "unknown");
        return 1;
    }
    g_object_get(plugin, "name", &name, "service", &service, NULL);
    CHECK(g_strcmp0(name, "NymVPN") == 0, "plugin name is not NymVPN");
    CHECK(g_strcmp0(service, SERVICE) == 0, "plugin service mismatch");
    CHECK(nm_vpn_editor_plugin_get_capabilities(plugin) == NM_VPN_EDITOR_PLUGIN_CAPABILITY_NONE,
          "capabilities must be NONE");
    g_free(name);
    g_free(service);

#ifdef WITH_GTK
    if (!gtk_init_check(
#if !GTK_CHECK_VERSION(4, 0, 0)
            NULL, NULL
#endif
            )) {
        printf("ok (editor skipped: no display)\n");
        return 0;
    }
    {
        NMConnection *connection = nm_simple_connection_new();
        NMSettingVpn *vpn = NM_SETTING_VPN(nm_setting_vpn_new());
        NMVpnEditor *editor, *again;
        GObject *widget;

        g_object_set(vpn, NM_SETTING_VPN_SERVICE_TYPE, SERVICE, NULL);
        nm_connection_add_setting(connection, NM_SETTING(vpn));

        editor = nm_vpn_editor_plugin_get_editor(plugin, connection, &error);
        if (!editor) {
            fprintf(stderr, "FAIL: get_editor: %s\n", error ? error->message : "unknown");
            return 1;
        }
        widget = nm_vpn_editor_get_widget(editor);
        CHECK(widget && GTK_IS_WIDGET(widget), "editor widget is not a GtkWidget");
        CHECK(nm_vpn_editor_update_connection(editor, connection, &error), "update_connection failed");
        CHECK(error == NULL, "update_connection set an error");

        again = nm_vpn_editor_plugin_get_editor(plugin, connection, &error);
        if (!again) {
            fprintf(stderr, "FAIL: second get_editor: %s\n", error ? error->message : "unknown");
            return 1;
        }
        g_object_unref(again);
        g_object_unref(editor);
        g_object_unref(connection);
    }
#endif

    g_object_unref(plugin);
    printf("ok\n");
    return 0;
}
