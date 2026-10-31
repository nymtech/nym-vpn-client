# Builds the NetworkManager editor plugin for NymVPN: one GTK-free core plugin and two editor
# libraries (GTK 3 for nm-connection-editor/nm-applet, GTK 4 for GNOME Settings).
CC ?= cc
PKG_CONFIG ?= pkg-config
BUILD ?= build
CFLAGS ?= -O2 -g
CFLAGS += -std=c11 -Wall -Wextra -fPIC -fvisibility=hidden -D_GNU_SOURCE

CORE := $(BUILD)/libnm-vpn-plugin-nymvpn.so
EDITOR_GTK3 := $(BUILD)/libnm-vpn-plugin-nymvpn-editor.so
EDITOR_GTK4 := $(BUILD)/libnm-gtk4-vpn-plugin-nymvpn-editor.so

LIBNM_CFLAGS := $(shell $(PKG_CONFIG) --cflags libnm gmodule-2.0)
LIBNM_LIBS := $(shell $(PKG_CONFIG) --libs libnm gmodule-2.0) -ldl
GTK3_CFLAGS := $(shell $(PKG_CONFIG) --cflags gtk+-3.0 gio-unix-2.0)
GTK3_LIBS := $(shell $(PKG_CONFIG) --libs gtk+-3.0 gio-unix-2.0)
GTK4_CFLAGS := $(shell $(PKG_CONFIG) --cflags gtk4 gio-unix-2.0)
GTK4_LIBS := $(shell $(PKG_CONFIG) --libs gtk4 gio-unix-2.0)

.PHONY: all test clean

# Libraries are 0644 like the installed files: libnm refuses group/other-writable plugins.
all: $(CORE) $(EDITOR_GTK3) $(EDITOR_GTK4)

$(BUILD):
	mkdir -p $(BUILD)

$(CORE): nm-nymvpn-editor-plugin.c | $(BUILD)
	$(CC) $(CFLAGS) $(LIBNM_CFLAGS) -shared -o $@ $< $(LIBNM_LIBS)
	chmod 644 $@

$(EDITOR_GTK3): nm-nymvpn-editor.c | $(BUILD)
	$(CC) $(CFLAGS) $(LIBNM_CFLAGS) $(GTK3_CFLAGS) -shared -o $@ $< $(LIBNM_LIBS) $(GTK3_LIBS)
	chmod 644 $@

$(EDITOR_GTK4): nm-nymvpn-editor.c | $(BUILD)
	$(CC) $(CFLAGS) $(LIBNM_CFLAGS) $(GTK4_CFLAGS) -shared -o $@ $< $(LIBNM_LIBS) $(GTK4_LIBS)
	chmod 644 $@

$(BUILD)/test-nogtk: test/load-plugin.c | $(BUILD)
	$(CC) $(CFLAGS) $(LIBNM_CFLAGS) -o $@ $< $(LIBNM_LIBS)

$(BUILD)/test-gtk3: test/load-plugin.c | $(BUILD)
	$(CC) $(CFLAGS) -DWITH_GTK $(LIBNM_CFLAGS) $(GTK3_CFLAGS) -o $@ $< $(LIBNM_LIBS) $(GTK3_LIBS)

$(BUILD)/test-gtk4: test/load-plugin.c | $(BUILD)
	$(CC) $(CFLAGS) -DWITH_GTK $(LIBNM_CFLAGS) $(GTK4_CFLAGS) -o $@ $< $(LIBNM_LIBS) $(GTK4_LIBS)

# The GTK tests run with no desktop entries and no PATH so nothing they do can open the real
# NymVPN app.
NO_APP := XDG_DATA_DIRS=/nonexistent XDG_DATA_HOME=/nonexistent XDG_CONFIG_HOME=/nonexistent PATH=/nonexistent
NAME_FILE := $(abspath ../packaging/nm-nymvpn-service.name)

test: $(BUILD)/test-nogtk $(BUILD)/test-gtk3 $(BUILD)/test-gtk4
	$(BUILD)/test-nogtk $(abspath $(CORE)) $(NAME_FILE)
	$(NO_APP) $(BUILD)/test-gtk3 $(abspath $(CORE))
	$(NO_APP) $(BUILD)/test-gtk4 $(abspath $(CORE))

clean:
	rm -rf $(BUILD)
