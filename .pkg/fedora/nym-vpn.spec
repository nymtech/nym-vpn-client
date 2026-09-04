%global debug_package %{nil}
%global _build_id_links none

Name:           nym-vpn
Version:        2026.11.2
Release:        1%{?dist}
Summary:        Decentralized, mixnet, and zero-knowledge VPN
License:        GPL-3.0-only
URL:            https://github.com/nymtech/nym-vpn-client
Source0:        %{url}/archive/refs/tags/nym-vpn-v%{version}.tar.gz
Source1:        net.nymtech.NymVPN.desktop
Source2:        net.nymtech.NymVPN.metainfo.xml
Source3:        nym-vpnd.service
Source4:        com.nymvpn.vpnd.unix-access.policy
Source5:        sources.lock

ExclusiveArch:  x86_64 aarch64

# The pinned container supplies newer upstream-required toolchains than Fedora's
# RPM database may expose. scripts/container-build.sh intentionally invokes
# rpmbuild --nodeps after installing all of these concrete build dependencies.
BuildRequires:  cargo >= 1.95
BuildRequires:  rust >= 1.95
BuildRequires:  golang >= 1.24.4
BuildRequires:  nodejs >= 24
BuildRequires:  npm
BuildRequires:  protobuf-compiler
BuildRequires:  gcc
BuildRequires:  gcc-c++
BuildRequires:  make
BuildRequires:  busybox
BuildRequires:  pkgconfig(dbus-1)
BuildRequires:  pkgconfig(gtk+-3.0)
BuildRequires:  pkgconfig(javascriptcoregtk-4.1)
BuildRequires:  pkgconfig(libmnl)
BuildRequires:  pkgconfig(libnftnl)
BuildRequires:  pkgconfig(openssl)
BuildRequires:  pkgconfig(webkit2gtk-4.1)
BuildRequires:  pkgconfig(ayatana-appindicator3-0.1)
BuildRequires:  pkgconfig(librsvg-2.0)
BuildRequires:  pkgconfig(libsoup-3.0)
BuildRequires:  desktop-file-utils
BuildRequires:  libappstream-glib
BuildRequires:  systemd-rpm-macros

Requires:       NetworkManager
Requires:       polkit
Requires:       webkit2gtk4.1
Requires:       gtk3
Requires:       dbus-libs
Requires:       openssl-libs
Requires:       libmnl
Requires:       libnftnl
Requires:       hicolor-icon-theme
%{?systemd_requires}

Provides:       nym-vpn-app = %{version}-%{release}
Provides:       nym-vpnd = %{version}-%{release}
Provides:       nym-vpnc = %{version}-%{release}
Provides:       nym-exclude = %{version}-%{release}
Provides:       nym-socks5-proxy = %{version}-%{release}
Provides:       nym-diagnostic = %{version}-%{release}

%description
NymVPN is a privacy-focused VPN supporting a five-hop mixnet mode and a
two-hop WireGuard mode. This combined package contains the graphical client,
daemon, command-line client, diagnostics, and daemon-managed helper programs.

This package targets Fedora 44 and is not an official Fedora distribution
package or Fedora-review submission.

%prep
echo "6d999fce5a83027aaccc71880f12a61ed2b74be6a38d70f0c0258d317c608463  %{SOURCE0}" | sha256sum --check --strict
rm -rf nym-vpn-client-nym-vpn-v%{version}
busybox tar -xzf %{SOURCE0}
chmod -Rf a+rX,u+w,g-w,o-w nym-vpn-client-nym-vpn-v%{version}
cd nym-vpn-client-nym-vpn-v%{version}

# Fail early if a retagged archive ever disagrees with the declared release.
grep -Fqx 'version = "2026.11.2"' nym-vpn-core/Cargo.toml
grep -Fqx 'version = "2026.11.2"' nym-vpn-app/src-tauri/Cargo.toml

%build
cd nym-vpn-client-nym-vpn-v%{version}
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export CARGO_BUILD_JOBS="%{_smp_build_ncpus}"
export CARGO_TARGET_DIR=/var/cache/nym-vpn-cargo-target/core
# redhat-rpm-config injects a second Rust optimization/debug profile. Upstream's
# release profile already enables LTO; using both greatly increases link memory
# and overrides upstream's strip setting without improving this binary RPM.
# aws-lc-sys treats the RPM environment's linker flags as compiler-probe
# arguments.  In particular, Fedora's default PIE link then breaks its
# deliberately non-PIE memcmp feature test.  Rust release profiles retain
# their own PIE, RELRO, and stack-protection defaults without these variables.
unset RUSTFLAGS LDFLAGS
unset SENTRY_DSN VPNLIB_SENTRY_DSN APP_SENTRY_DSN
export DEV_MODE=false
export UPDATER_ENABLED=false

./wireguard/build-wireguard-go.sh

pushd nym-vpn-core
for package in \
    nym-vpnd \
    nym-vpnc \
    nym-exclude \
    nym-socks5-proxy \
    nym-diagnostic; do
    cargo build --release --locked -p "${package}"
done
popd

export CARGO_TARGET_DIR=/var/cache/nym-vpn-cargo-target/app
pushd nym-vpn-app
npm ci
npm run gen:licenses:js
npm run gen:licenses:rust
npm run tauri build -- --no-bundle
popd

%install
cd nym-vpn-client-nym-vpn-v%{version}
install -Dpm0755 /var/cache/nym-vpn-cargo-target/app/release/nym-vpn-app \
    %{buildroot}%{_bindir}/nym-vpn-app
install -Dpm0755 /var/cache/nym-vpn-cargo-target/core/release/nym-vpnd \
    %{buildroot}%{_bindir}/nym-vpnd
install -Dpm0755 /var/cache/nym-vpn-cargo-target/core/release/nym-vpnc \
    %{buildroot}%{_bindir}/nym-vpnc
install -Dpm4755 /var/cache/nym-vpn-cargo-target/core/release/nym-exclude \
    %{buildroot}%{_bindir}/nym-exclude
install -Dpm0755 /var/cache/nym-vpn-cargo-target/core/release/nym-socks5-proxy \
    %{buildroot}%{_bindir}/nym-socks5-proxy
install -Dpm0755 /var/cache/nym-vpn-cargo-target/core/release/nym-diagnostic \
    %{buildroot}%{_bindir}/nym-diagnostic

install -Dpm0644 %{SOURCE1} \
    %{buildroot}%{_datadir}/applications/net.nymtech.NymVPN.desktop
install -Dpm0644 %{SOURCE2} \
    %{buildroot}%{_metainfodir}/net.nymtech.NymVPN.metainfo.xml
install -Dpm0644 %{SOURCE3} \
    %{buildroot}%{_unitdir}/nym-vpnd.service
install -Dpm0644 %{SOURCE4} \
    %{buildroot}%{_datadir}/polkit-1/actions/com.nymvpn.vpnd.unix-access.policy
install -Dpm0644 nym-vpn-app/public/icon.svg \
    %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/nym-vpn.svg

install -Dpm0644 nym-vpn-app/public/licenses-js.json \
    %{buildroot}%{_docdir}/%{name}/licenses-js.json
install -Dpm0644 nym-vpn-app/public/licenses-rust.json \
    %{buildroot}%{_docdir}/%{name}/licenses-rust.json

desktop-file-validate \
    %{buildroot}%{_datadir}/applications/net.nymtech.NymVPN.desktop
appstreamcli validate --no-net \
    %{buildroot}%{_metainfodir}/net.nymtech.NymVPN.metainfo.xml

%post
%systemd_post nym-vpnd.service
if [ "$1" -eq 1 ] && [ -d /run/systemd/system ]; then
    systemctl enable --now nym-vpnd.service >/dev/null 2>&1 || :
fi

%preun
%systemd_preun nym-vpnd.service

%postun
%systemd_postun_with_restart nym-vpnd.service

%files
%license nym-vpn-client-nym-vpn-v%{version}/LICENSE
%doc nym-vpn-client-nym-vpn-v%{version}/README.md
%doc %{_docdir}/%{name}/licenses-js.json
%doc %{_docdir}/%{name}/licenses-rust.json
%{_bindir}/nym-vpn-app
%{_bindir}/nym-vpnd
%{_bindir}/nym-vpnc
%attr(4755,root,root) %{_bindir}/nym-exclude
%{_bindir}/nym-socks5-proxy
%{_bindir}/nym-diagnostic
%{_unitdir}/nym-vpnd.service
%{_datadir}/polkit-1/actions/com.nymvpn.vpnd.unix-access.policy
%{_datadir}/applications/net.nymtech.NymVPN.desktop
%{_datadir}/icons/hicolor/scalable/apps/nym-vpn.svg
%{_metainfodir}/net.nymtech.NymVPN.metainfo.xml

%changelog
* Wed Jul 15 2026 NymVPN contributors <contact@nymtech.net> - 2026.11.2-1
- Build the aligned NymVPN GUI and core release as one Fedora 44 package
- Add x86_64 and aarch64 container builds and validation
