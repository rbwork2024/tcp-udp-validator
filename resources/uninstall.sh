#!/bin/sh

systemctl disable --now tcp-udp-validator.service

rm -f /lib/systemd/system/tcp-udp-validator.service
rm -f /usr/bin/tcp-udp-validator
