{{- define "jms.tpl.fullname" -}}
{{- template "jms.fullname" . -}}-{{- .name -}}
{{- end -}}

{{- define "jms.tpl.name" -}}
{{- template "jms.name" . -}}-{{- .name -}}
{{- end -}}


{{- define "jms.tpl.container.spec" -}}
image: {{ default .Values.image .valspec.image }}:{{ default .Values.tag .valspec.tag }}
imagePullPolicy: {{ default .Values.imagePullPolicy .valspec.imagePullPolicy }}
env:
- name: REDIS_URI
  value: "redis://{{ template "common.names.fullname" .Subcharts.redis }}-master:6379/0"
- name: RABBITMQ_URI
  value: "amqp://user:rabbitmq@{{ template "common.names.fullname" .Subcharts.rabbitmq }}:5672/%2f"
{{- with .valspec.container }}
{{- toYaml . | nindent 0 }}
{{- end -}}
{{- end -}}