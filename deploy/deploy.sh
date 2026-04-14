#!/usr/bin/env bash
# 코드 업데이트 시 재배포 스크립트
# 사용법: VM에 SSH 접속 후 프로젝트 디렉토리에서 ./deploy/deploy.sh
set -e

cd "$(dirname "$0")/.."

echo "=== git pull ==="
git pull origin main

echo "=== docker compose build ==="
cd deploy
docker compose build commitcat-api

echo "=== docker compose up (서버만 재기동, nginx는 유지) ==="
docker compose up -d --no-deps commitcat-api

echo "=== 헬스체크 ==="
sleep 3
docker compose exec commitcat-api wget -qO- http://localhost:3000/health || echo "WARN: 헬스체크 실패"

echo "완료."
