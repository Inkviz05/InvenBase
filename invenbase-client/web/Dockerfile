FROM node:22-alpine AS builder

WORKDIR /app

COPY invenbase-client/web/package.json invenbase-client/web/package-lock.json ./
RUN npm ci

COPY invenbase-client/web/index.html ./
COPY invenbase-client/web/public ./public
COPY invenbase-client/web/src ./src
COPY invenbase-client/web/vite.config.js ./

ARG VITE_API_URL=/api
ENV VITE_API_URL=${VITE_API_URL}

RUN npm run build

FROM nginx:1.27-alpine AS runtime

COPY invenbase-client/web/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://127.0.0.1/health || exit 1
