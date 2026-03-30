#!/bin/sh
# api/web runtime: dış internete giden yeni bağlantıları reddeder; Docker iç ağı, DNS stub ve loopback kalır.
# Gerekli: imajda iptables + compose'ta cap_add: NET_ADMIN
set -eu

# node:*-slim imaji PATH'te /usr/sbin yok; iptables paketi binary'yi /usr/sbin/iptables'a koyar.
IPT=""
for candidate in /usr/sbin/iptables /sbin/iptables; do
  if [ -x "$candidate" ]; then
    IPT=$candidate
    break
  fi
done

if [ -z "$IPT" ]; then
  echo "container-block-wan-egress: iptables bulunamadi (imaj: apt install iptables; ardindan docker compose build --no-cache web api)" >&2
  exit 1
fi

if "$IPT" -N KORO_EGRESS 2>/dev/null; then
  :
else
  "$IPT" -F KORO_EGRESS
fi

if ! "$IPT" -A KORO_EGRESS -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT 2>/dev/null; then
  "$IPT" -A KORO_EGRESS -m state --state ESTABLISHED,RELATED -j ACCEPT
fi
"$IPT" -A KORO_EGRESS -o lo -j ACCEPT
"$IPT" -A KORO_EGRESS -p udp -d 127.0.0.11 --dport 53 -j ACCEPT
"$IPT" -A KORO_EGRESS -p tcp -d 127.0.0.11 --dport 53 -j ACCEPT
"$IPT" -A KORO_EGRESS -d 10.0.0.0/8 -j ACCEPT
"$IPT" -A KORO_EGRESS -d 172.16.0.0/12 -j ACCEPT
"$IPT" -A KORO_EGRESS -d 192.168.0.0/16 -j ACCEPT
"$IPT" -A KORO_EGRESS -d 127.0.0.0/8 -j ACCEPT
"$IPT" -A KORO_EGRESS -d 169.254.0.0/16 -j ACCEPT
"$IPT" -A KORO_EGRESS -j REJECT --reject-with icmp-port-unreachable

if ! "$IPT" -C OUTPUT -j KORO_EGRESS 2>/dev/null; then
  "$IPT" -I OUTPUT 1 -j KORO_EGRESS
fi

IP6T=""
for candidate in /usr/sbin/ip6tables /sbin/ip6tables; do
  if [ -x "$candidate" ]; then
    IP6T=$candidate
    break
  fi
done
if [ -n "$IP6T" ]; then
  "$IP6T" -P OUTPUT DROP 2>/dev/null || true
fi

exec "$@"
