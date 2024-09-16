#!/bin/sh

install -Dm0755 tcp-udp-validator /usr/bin/tcp-udp-validator
install -Dm0644 tcp-udp-validator.service /lib/systemd/system/tcp-udp-validator.service

systemctl enable --now tcp-udp-validator.service