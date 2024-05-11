firewall --disabled
keyboard 'us'
lang en_AU.UTF-8

timezone Australia/Sydney --utc

network --bootproto=static --ip 10.0.100.10 --netmask 255.255.255.0 --gateway 10.0.100.1 --nameserver=10.0.100.1,8.8.8.8 --device=link --activate --hostname=jms-master.jms.local

reboot

repo --name="BaseOS" --baseurl=http://dl.rockylinux.org/pub/rocky/9/BaseOS/$basearch/os/ --cost=200
repo --name="AppStream" --baseurl=http://dl.rockylinux.org/pub/rocky/9/AppStream/$basearch/os/ --cost=200
repo --name="CRB" --baseurl=http://dl.rockylinux.org/pub/rocky/9/CRB/$basearch/os/ --cost=200
repo --name="extras" --baseurl=http://dl.rockylinux.org/pub/rocky/9/extras/$basearch/os --cost=200

zerombr
clearpart --all --initlabel
autopart

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

%post --erroronfail
echo "PATH=\"/usr/local/bin:/usr/local/sbin:/usr/bin:/usr/sbin:/bin\"" > /etc/environment

# Install RKE2
mkdir -p /etc/rancher/rke2
echo "token: jmsR0cks" > /etc/rancher/rke2/config.yaml
curl -sfL https://get.rke2.io | INSTALL_RKE2_TYPE=server sh -
systemctl enable rke2-server.service

# Everything else has to happen after startup
cat > /usr/local/bin/jms-provision.sh <<'EOF'
#!/bin/sh

set -e

ln -s $(find /var/lib/rancher/rke2/data/ -name kubectl) /usr/local/bin/kubectl

# Install Helm
curl -L https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# Propagate KUBECONFIG
mkdir -p /home/fta/.kube
cp /etc/rancher/rke2/rke2.yaml /home/fta/.kube/config
chown fta /home/fta/.kube/config
mkdir -p /root/.kube
cp /etc/rancher/rke2/rke2.yaml /root/.kube/config
export KUBECONFIG=/root/.kube/config

# Install Rancher
helm repo add rancher-latest https://releases.rancher.com/server-charts/latest
helm repo add jetstack https://charts.jetstack.io
kubectl apply -f https://github.com/jetstack/cert-manager/releases/download/v1.6.1/cert-manager.crds.yaml
helm upgrade -i cert-manager jetstack/cert-manager --namespace cert-manager --create-namespace
helm upgrade -i rancher rancher-latest/rancher --create-namespace --namespace cattle-system --set hostname=jms-master.jms.local --set bootstrapPassword=jmsR0cks --set replicas=1

# Install Longhorn, MetalLB
helm repo add longhorn https://charts.longhorn.io
helm repo update
helm upgrade -i longhorn longhorn/longhorn --namespace longhorn-system --create-namespace
kubectl apply -f https://raw.githubusercontent.com/metallb/metallb/v0.14.3/config/manifests/metallb-native.yaml

# Disable the provisioning service, just so we don't run twice
systemctl disable jms-provision.service
EOF

chown root:root /usr/local/bin/jms-provision.sh
chmod u+rx /usr/local/bin/jms-provision.sh

# Make it trigger at startup
cat > /etc/systemd/system/jms-provision.service <<EOF
[Unit]
Description=JMS Provisioning Service
After=network.target
After=systemd-user-sessions.service
After=network-online.target
After=rke2-server.service

[Service]
ExecStart=/usr/local/bin/jms-provision.sh
Restart=no
Type=oneshot
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

systemctl enable jms-provision.service

%end
