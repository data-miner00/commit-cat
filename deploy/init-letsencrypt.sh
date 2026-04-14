#!/usr/bin/env bash
# Let's Encrypt 최초 인증서 발급 스크립트
# 사용법: ./init-letsencrypt.sh
# 사전 조건: .env 파일에 DOMAIN, EMAIL 설정 완료
set -e

if [ ! -f .env ]; then
    echo "ERROR: .env 파일이 없습니다. .env.example을 복사해서 만드세요."
    exit 1
fi

# shellcheck disable=SC1091
source .env

if [ -z "$DOMAIN" ] || [ -z "$EMAIL" ]; then
    echo "ERROR: .env에 DOMAIN과 EMAIL을 설정하세요."
    exit 1
fi

# nginx/default.conf의 YOUR_DOMAIN을 실제 도메인으로 치환
sed -i.bak "s/YOUR_DOMAIN/$DOMAIN/g" nginx/default.conf
rm -f nginx/default.conf.bak

echo "=== 1. 임시 self-signed 인증서로 nginx 먼저 띄우기 ==="
mkdir -p certbot/conf/live/"$DOMAIN"
openssl req -x509 -nodes -newkey rsa:2048 -days 1 \
    -keyout certbot/conf/live/"$DOMAIN"/privkey.pem \
    -out certbot/conf/live/"$DOMAIN"/fullchain.pem \
    -subj "/CN=localhost"

echo "=== 2. nginx 기동 ==="
docker compose up -d nginx

echo "=== 3. 임시 인증서 제거 ==="
rm -rf certbot/conf/live/"$DOMAIN"

echo "=== 4. Let's Encrypt 실제 인증서 발급 ==="
docker compose run --rm certbot certonly \
    --webroot -w /var/www/certbot \
    --email "$EMAIL" \
    -d "$DOMAIN" \
    --agree-tos \
    --no-eff-email \
    --force-renewal

echo "=== 5. nginx reload ==="
docker compose exec nginx nginx -s reload

echo "=== 6. 전체 스택 기동 ==="
docker compose up -d

echo "완료. https://$DOMAIN 에서 확인하세요."
