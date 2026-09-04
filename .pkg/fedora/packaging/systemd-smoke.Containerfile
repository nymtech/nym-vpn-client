FROM fedora:44

RUN dnf -y --setopt=install_weak_deps=False install \
        NetworkManager \
        dbus-daemon \
        polkit \
        systemd \
        util-linux && \
    dnf clean all && \
    ln -sfn /usr/lib/systemd/system/dbus-daemon.service \
        /etc/systemd/system/dbus.service

STOPSIGNAL SIGRTMIN+3
CMD ["/usr/lib/systemd/systemd"]
