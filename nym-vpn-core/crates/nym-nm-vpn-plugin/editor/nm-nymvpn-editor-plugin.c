/* Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
 * SPDX-License-Identifier: GPL-3.0-only
 *
 * Core NetworkManager editor plugin for NymVPN.
 *
 * GTK-free, so nmcli can load it. It describes the plugin to libnm clients and, when a client
 * asks for an editor, loads the editor library matching the client's toolkit from the directory
 * this library lives in.
 */
#include <dlfcn.h>
#include <gmodule.h>
#include <string.h>

#include <NetworkManager.h>

#define NYMVPN_SERVICE "org.freedesktop.NetworkManager.nymvpn"
#define NYMVPN_PLUGIN_NAME "NymVPN"
#define NYMVPN_PLUGIN_DESC "Nym mixnet VPN"

#define EDITOR_GTK3 "libnm-vpn-plugin-nymvpn-editor.so"
#define EDITOR_GTK4 "libnm-gtk4-vpn-plugin-nymvpn-editor.so"
#define EDITOR_FACTORY "nm_vpn_editor_factory_nymvpn"

typedef NMVpnEditor *(*NymvpnEditorFactory)(NMVpnEditorPlugin *plugin,
                                            NMConnection *connection,
                                            GError **error);

#define NYMVPN_TYPE_EDITOR_PLUGIN (nymvpn_editor_plugin_get_type())
G_DECLARE_FINAL_TYPE(NymvpnEditorPlugin, nymvpn_editor_plugin, NYMVPN, EDITOR_PLUGIN, GObject)

struct _NymvpnEditorPlugin {
    GObject parent;
};

enum { PROP_0, PROP_NAME, PROP_DESC, PROP_SERVICE };

static void nymvpn_editor_plugin_interface_init(NMVpnEditorPluginInterface *iface);

G_DEFINE_TYPE_EXTENDED(NymvpnEditorPlugin, nymvpn_editor_plugin, G_TYPE_OBJECT, 0,
                       G_IMPLEMENT_INTERFACE(NM_TYPE_VPN_EDITOR_PLUGIN,
                                             nymvpn_editor_plugin_interface_init))

/* The editor library stays loaded for the life of the process: it registers GTypes that cannot
 * be unregistered. */
static NymvpnEditorFactory cached_factory;
static char *cached_editor_name;

static char *editor_path(const char *editor_name)
{
    Dl_info info;

    if (!dladdr((void *) editor_path, &info) || !info.dli_fname)
        return NULL;
    {
        char *dir = g_path_get_dirname(info.dli_fname);
        char *path = g_build_filename(dir, editor_name, NULL);
        g_free(dir);
        return path;
    }
}

static gboolean host_is_gtk3(void)
{
    GModule *self = g_module_open(NULL, 0);
    gpointer symbol = NULL;

    if (self) {
        g_module_symbol(self, "gtk_container_add", &symbol);
        g_module_close(self);
    }
    return symbol != NULL;
}

static NymvpnEditorFactory load_editor_factory(const char *editor_name, GError **error)
{
    char *path;
    void *module;
    void *symbol;

    if (cached_factory) {
        if (g_strcmp0(cached_editor_name, editor_name) != 0) {
            g_set_error(error, NM_VPN_PLUGIN_ERROR, NM_VPN_PLUGIN_ERROR_FAILED,
                        "editor %s already loaded, cannot load %s in the same process",
                        cached_editor_name, editor_name);
            return NULL;
        }
        return cached_factory;
    }

    path = editor_path(editor_name);
    if (!path) {
        g_set_error(error, NM_VPN_PLUGIN_ERROR, NM_VPN_PLUGIN_ERROR_FAILED,
                    "cannot locate the directory of the NymVPN plugin");
        return NULL;
    }
    if (!g_file_test(path, G_FILE_TEST_EXISTS)) {
        g_set_error(error, G_FILE_ERROR, G_FILE_ERROR_NOENT, "missing plugin file \"%s\"", path);
        g_free(path);
        return NULL;
    }
    module = dlopen(path, RTLD_LAZY | RTLD_LOCAL);
    if (!module) {
        g_set_error(error, NM_VPN_PLUGIN_ERROR, NM_VPN_PLUGIN_ERROR_FAILED,
                    "cannot load editor plugin %s: %s", path, dlerror());
        g_free(path);
        return NULL;
    }
    symbol = dlsym(module, EDITOR_FACTORY);
    if (!symbol) {
        g_set_error(error, NM_VPN_PLUGIN_ERROR, NM_VPN_PLUGIN_ERROR_FAILED,
                    "cannot load factory %s from %s: %s", EDITOR_FACTORY, path, dlerror());
        dlclose(module);
        g_free(path);
        return NULL;
    }
    g_free(path);

    cached_factory = (NymvpnEditorFactory) symbol;
    cached_editor_name = g_strdup(editor_name);
    return cached_factory;
}

static NMVpnEditor *get_editor(NMVpnEditorPlugin *plugin, NMConnection *connection, GError **error)
{
    NymvpnEditorFactory factory;
    NMVpnEditor *editor;

    g_return_val_if_fail(NYMVPN_IS_EDITOR_PLUGIN(plugin), NULL);
    g_return_val_if_fail(NM_IS_CONNECTION(connection), NULL);
    g_return_val_if_fail(!error || !*error, NULL);

    factory = load_editor_factory(host_is_gtk3() ? EDITOR_GTK3 : EDITOR_GTK4, error);
    if (!factory)
        return NULL;

    editor = factory(plugin, connection, error);
    if (!editor && error && !*error) {
        g_set_error(error, NM_VPN_PLUGIN_ERROR, NM_VPN_PLUGIN_ERROR_FAILED,
                    "unknown error creating the NymVPN editor");
    }
    return editor;
}

static NMVpnEditorPluginCapability get_capabilities(NMVpnEditorPlugin *plugin)
{
    (void) plugin;
    return NM_VPN_EDITOR_PLUGIN_CAPABILITY_NONE;
}

static void get_property(GObject *object, guint prop_id, GValue *value, GParamSpec *pspec)
{
    switch (prop_id) {
    case PROP_NAME:
        g_value_set_string(value, NYMVPN_PLUGIN_NAME);
        break;
    case PROP_DESC:
        g_value_set_string(value, NYMVPN_PLUGIN_DESC);
        break;
    case PROP_SERVICE:
        g_value_set_string(value, NYMVPN_SERVICE);
        break;
    default:
        G_OBJECT_WARN_INVALID_PROPERTY_ID(object, prop_id, pspec);
        break;
    }
}

static void nymvpn_editor_plugin_init(NymvpnEditorPlugin *self)
{
    (void) self;
}

static void nymvpn_editor_plugin_class_init(NymvpnEditorPluginClass *klass)
{
    GObjectClass *object_class = G_OBJECT_CLASS(klass);

    object_class->get_property = get_property;
    g_object_class_override_property(object_class, PROP_NAME, NM_VPN_EDITOR_PLUGIN_NAME);
    g_object_class_override_property(object_class, PROP_DESC, NM_VPN_EDITOR_PLUGIN_DESCRIPTION);
    g_object_class_override_property(object_class, PROP_SERVICE, NM_VPN_EDITOR_PLUGIN_SERVICE);
}

static void nymvpn_editor_plugin_interface_init(NMVpnEditorPluginInterface *iface)
{
    iface->get_editor = get_editor;
    iface->get_capabilities = get_capabilities;
}

/* Explicit visibility: the library is built with -fvisibility=hidden and older GLib headers do
 * not guarantee G_MODULE_EXPORT carries the attribute. */
__attribute__((visibility("default"))) G_MODULE_EXPORT NMVpnEditorPlugin *
nm_vpn_editor_plugin_factory(GError **error)
{
    g_return_val_if_fail(!error || !*error, NULL);
    return g_object_new(NYMVPN_TYPE_EDITOR_PLUGIN, NULL);
}
