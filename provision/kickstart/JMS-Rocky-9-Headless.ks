text

firewall --disabled
keyboard 'us'
lang en_AU.UTF-8

timezone Australia/Sydney --utc

%include /tmp/jms-config

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

%pre --erroronfail --interpreter=/usr/bin/bash
exec < /dev/tty6 > /dev/tty6 2>&1
chvt 6

TYPE=""

until [[ "$TYPE" == "primary" ]] || [[ "$TYPE" == "secondary" ]] || [[ "$TYPE" == "tertiary" ]]; do
  echo
  echo " *** Select JMS Node Type -- primary, secondary, or teritary *** "
  echo
  read -p "Type: " TYPE
done

echo "$TYPE" > /tmp/jms-type

if [[ "$TYPE" == "primary" ]]; then
  echo "* PRIMARY selected"
  echo "network --bootproto=static --ip 10.0.100.10 --netmask 255.255.255.0 --gateway 10.0.100.1 --nameserver=10.0.100.1,8.8.8.8 --device=link --activate --hostname=jms-primary.jms.local" > /tmp/jms-config
elif [[ "$TYPE" == "secondary" ]]; then
  echo "* SECONDARY selected"
  echo "network --bootproto=static --ip 10.0.100.11 --netmask 255.255.255.0 --gateway 10.0.100.1 --nameserver=10.0.100.1,8.8.8.8 --device=link --activate --hostname=jms-secondary.jms.local" > /tmp/jms-config
elif [[ "$TYPE" == "tertiary" ]]; then
  echo "* TERTIARY selected"
  echo "network --bootproto=static --ip 10.0.100.12 --netmask 255.255.255.0 --gateway 10.0.100.1 --nameserver=10.0.100.1,8.8.8.8 --device=link --activate --hostname=jms-tertiary.jms.local" > /tmp/jms-config
fi

chvt 1
%end

# Copy jms-type to the system
%post --erroronfail --nochroot
exec < /dev/tty6 > /dev/tty6 2>&1
chvt 6
echo "JMS Type: $(cat /tmp/jms-type)"

echo "[#] Extending Root Partition"
lvextend $(find /dev/mapper -name "*-root") -l +100%FREE -r

# Make the sysimage aware of the JMS type
cp /tmp/jms-type /mnt/sysimage/etc/jms-type

# Copy through the additional stuff
cp -r /run/install/repo/k8s /mnt/sysimage/home/fta
cp -r /run/install/repo/docker_images /mnt/sysimage/home/fta

chvt 1
%end

# Install RKE2, install post-restart provisioning script
%post --erroronfail
exec < /dev/tty6 > /dev/tty6 2>&1
chvt 6

echo "PATH=\"/usr/local/bin:/usr/local/sbin:/usr/bin:/usr/sbin:/bin\"" > /etc/environment

echo "[#] Installing Docker"
dnf config-manager --add-repo https://download.docker.com/linux/centos/docker-ce.repo
dnf -y install docker-ce docker-ce-cli containerd.io docker-compose-plugin
systemctl enable docker
usermod -a -G docker fta

echo "[#] Installing RKE2..."

# Install RKE2
export JMS_TYPE=$(cat /etc/jms-type)
mkdir -p /etc/rancher/rke2
cat > /etc/rancher/rke2/config.yaml <<'EOF'
token: jmsR0cks
cluster-cidr: 172.16.0.0/16
service-cidr: 172.17.0.0/16
cluster-dns: 172.17.0.10
cni:
- multus
- canal
EOF

# If we're not a primary node, make sure we're connecting to the same
# cluster as the primary.
if [[ "$JMS_TYPE" != "primary" ]]; then
cat >> /etc/rancher/rke2/config.yaml <<'EOF'
server: 10.0.100.10
EOF
fi

curl -sfL https://get.rke2.io | INSTALL_RKE2_TYPE=server sh -
systemctl enable rke2-server.service

# Everything else has to happen after startup
echo "[#] Writing Provision Script..."
cat > /usr/local/bin/jms-provision.sh <<'EOF'
#!/bin/sh

set -e

JMS_TYPE=$(cat /etc/jms-type)

ln -s $(find /var/lib/rancher/rke2/data/ -name kubectl) /usr/local/bin/kubectl || true

# Install Helm
curl -L https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# Propagate KUBECONFIG
mkdir -p /home/fta/.kube
cp /etc/rancher/rke2/rke2.yaml /home/fta/.kube/config
chown fta /home/fta/.kube/config
mkdir -p /root/.kube
cp /etc/rancher/rke2/rke2.yaml /root/.kube/config
export KUBECONFIG=/root/.kube/config

# Only need to install Rancher, Longhorn if we're not the primary - otherwise it gets propoagated by the primary
if [[ "$JMS_TYPE" == "primary" ]]; then
# Install Rancher
helm repo add rancher-latest https://releases.rancher.com/server-charts/latest
helm repo add jetstack https://charts.jetstack.io
kubectl apply -f https://github.com/jetstack/cert-manager/releases/download/v1.6.1/cert-manager.crds.yaml
helm upgrade -i cert-manager jetstack/cert-manager --namespace cert-manager --create-namespace
helm upgrade -i rancher rancher-latest/rancher --create-namespace --namespace cattle-system --set hostname=$(hostname) --set bootstrapPassword=jmsR0cks --set replicas=1

# Install Longhorn, MetalLB
helm repo add longhorn https://charts.longhorn.io
helm repo update
helm upgrade -i longhorn longhorn/longhorn --namespace longhorn-system --create-namespace
kubectl apply -f https://raw.githubusercontent.com/metallb/metallb/v0.14.3/config/manifests/metallb-native.yaml
fi

# Install CNI
cp /home/fta/k8s/jms-cni /opt/cni/bin/jms-cni

# Install Images
/var/lib/rancher/rke2/bin/ctr --address /run/k3s/containerd/containerd.sock --namespace k8s.io image import /home/fta/docker_images/jms.tar
/var/lib/rancher/rke2/bin/ctr --address /run/k3s/containerd/containerd.sock --namespace k8s.io image import /home/fta/docker_images/jms-ui.tar

# Disable the provisioning service, just so we don't run twice
systemctl disable jms-provision.service
EOF

chown root:root /usr/local/bin/jms-provision.sh
chmod u+rx /usr/local/bin/jms-provision.sh

# Make it trigger at startup. Need restart as sometimes rke2 isn't ready yet.
cat > /etc/systemd/system/jms-provision.service <<EOF
[Unit]
Description=JMS Provisioning Service
After=network.target
After=systemd-user-sessions.service
After=network-online.target
After=rke2-server.service

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
