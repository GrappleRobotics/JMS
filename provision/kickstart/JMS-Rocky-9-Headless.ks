text

firewall --disabled
keyboard 'us'
lang en_AU.UTF-8

timezone Australia/Sydney --utc

# %include /tmp/jms-config
network --bootproto=static --ip 10.0.100.10 --netmask 255.255.255.0 --gateway 10.0.100.1 --nameserver=10.0.100.1,8.8.8.8 --device=link --activate --hostname=jms-host.jms.local

reboot

repo --name="BaseOS" --baseurl=http://dl.rockylinux.org/pub/rocky/9/BaseOS/$basearch/os/ --cost=200
repo --name="AppStream" --baseurl=http://dl.rockylinux.org/pub/rocky/9/AppStream/$basearch/os/ --cost=200
repo --name="CRB" --baseurl=http://dl.rockylinux.org/pub/rocky/9/CRB/$basearch/os/ --cost=200
repo --name="extras" --baseurl=http://dl.rockylinux.org/pub/rocky/9/extras/$basearch/os --cost=200

zerombr
clearpart --all --initlabel
autopart --nohome

selinux --disabled

rootpw --lock
user --groups=wheel --name=fta --plaintext --password=jmsR0cks --gecos="FTA"

eula --agreed

%packages
git
nfs-utils
cryptsetup
iscsi-initiator-utils
tar
%end

# %pre --erroronfail --interpreter=/usr/bin/bash
# exec < /dev/tty6 > /dev/tty6 2>&1
# chvt 6

# chvt 1
# %end

# Copy jms-type to the system
%post --erroronfail --nochroot
exec < /dev/tty6 > /dev/tty6 2>&1
chvt 6

echo "[#] Extending Root Partition"
lvextend $(find /dev/mapper -name "*-root") -l +100%FREE -r

echo "[#] Writing JMS Env Vars"
mkdir -p /mnt/sysimage/jms/data
mkdir -p /mnt/sysimage/jms/data/redis
mkdir -p /mnt/sysimage/jms/data/unifi

TARGET_INTERFACE=$(ip -brief addr show | grep -i 10.0.100. | awk '{print $1}')
echo "JMS_INTERFACE=$TARGET_INTERFACE" >> /mnt/sysimage/jms/.env
echo "JMS_DATA_DIR=/jms/data" >> /mnt/sysimage/jms/.env

# Copy through the additional stuff
cp -r /run/install/repo/docker_images /mnt/sysimage/jms
cp /run/install/repo/docker_images/docker-compose.yml /mnt/sysimage/jms/docker-compose.yml

chvt 1
%end

# Install RKE2, install post-restart provisioning script
%post --erroronfail
exec < /dev/tty6 > /dev/tty6 2>&1
chvt 6

mkdir -p /jms/data

echo "PATH=\"/usr/local/bin:/usr/local/sbin:/usr/bin:/usr/sbin:/bin\"" > /etc/environment

echo "[#] Installing Docker"
dnf config-manager --add-repo https://download.docker.com/linux/centos/docker-ce.repo
dnf -y install docker-ce docker-ce-cli containerd.io docker-compose-plugin
systemctl enable docker
usermod -a -G docker fta

# Everything else has to happen after startup
echo "[#] Writing Provision Script..."
cat > /usr/local/bin/jms-provision.sh <<'EOF'
#!/bin/sh

set -e

# Install Images
docker image load --input /jms/docker_images/jms.tar
docker image load --input /jms/docker_images/jms-ui.tar
docker image load --input /jms/docker_images/jms-nginx.tar

# Disable the provisioning service, just so we don't run twice
systemctl disable jms-provision.service
EOF

chown root:root /usr/local/bin/jms-provision.sh
chmod u+rx /usr/local/bin/jms-provision.sh
chown -R fta:fta /jms

cat > /etc/systemd/system/jms-provision.service <<EOF
[Unit]
Description=JMS Provisioning Service
After=network.target
After=systemd-user-sessions.service
After=network-online.target
After=docker.service

[Service]
ExecStart=/usr/local/bin/jms-provision.sh
Restart=on-failure
RestartSec=30
Type=oneshot
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

systemctl enable jms-provision.service

echo "[#] Done!"

chvt 1
%end
