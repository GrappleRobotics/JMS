{{- define "jms.statefulset" -}}
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: {{ include "jms.tpl.fullname" . }}
  labels:
    app: {{ include "jms.tpl.name" . }}
    chart: {{ include "jms.chart" . }}
    release: {{ .Release.Name | quote }}
spec:
  selector:
    matchLabels:
      app: {{ include "jms.tpl.name" . }}
      release: {{ .Release.Name | quote }}
  replicas: 1
  serviceName: {{ .name }}
  template:
    metadata:
      labels:
        app: {{ include "jms.tpl.name" . }}
        chart: {{ include "jms.chart" . }}
        release: {{ .Release.Name | quote }}
    spec:
      imagePullSecrets:
        {{ toYaml .Values.imagePullSecrets | nindent 8 }}
      containers:
      - name: {{ include "jms.tpl.fullname" . }}
        command: {{ .valspec.command }}
        args: {{ .valspec.args }}
        resources:
          {{ toYaml .valspec.resources | nindent 10 }}
        {{ include "jms.tpl.container.spec" . | nindent 8 }}
        volumeMounts:
        - name: tz-config
          mountPath: /etc/localtime
      volumes:
      - name: tz-config
        hostPath:
          path: /etc/localtime
          type: File

{{- end -}}
