FROM NGINX1.20_DOCKER_REGISTRY_URL

user root

COPY dist/spa/ /usr/share/nginx/html
COPY nginx.template.conf /etc/nginx/nginx.conf
