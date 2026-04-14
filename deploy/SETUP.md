# Oracle Cloud Always Free 배포 가이드

commitcat-api를 Oracle Cloud의 Ampere A1 VM (영구 무료) 에 배포합니다.

---

## 0. 준비물

- Oracle Cloud 계정 (신용카드 필요하지만 **과금 0**)
- 터미널 기본 지식 (ssh 접속 가능)

---

## 1. Oracle Cloud VM 만들기

### 1.1 회원가입
https://signup.oraclecloud.com → 카드 등록 (검증용, 자동 과금 없음)

### 1.2 Compute Instance 생성
왼쪽 햄버거 메뉴 → **Compute** → **Instances** → **Create Instance**

| 항목 | 값 |
|---|---|
| Name | `commitcat-api` |
| Image | **Canonical Ubuntu 22.04** |
| Shape | **Change shape** → **Ampere** → `VM.Standard.A1.Flex` |
| OCPUs | 2 (최대 4까지 무료) |
| Memory | 12 GB (최대 24 GB까지 무료) |
| Networking | 기본값 유지 (VCN 자동 생성, public IP 할당) |
| SSH keys | **Generate a key pair** → private key **반드시 다운로드** |

**Create** 클릭 후 ~2분 대기.

### 1.3 방화벽 열기 (Security List)

Networking → Virtual Cloud Networks → 생성된 VCN 클릭 → Security Lists → Default Security List → **Add Ingress Rules**

| Source | Protocol | Port |
|---|---|---|
| 0.0.0.0/0 | TCP | 80 |
| 0.0.0.0/0 | TCP | 443 |

### 1.4 Public IP 확인
Instance 상세 페이지에서 **Public IP** 복사 (예: `132.145.xx.xx`)

---

## 2. VM 초기 설정 (로컬 맥에서)

```bash
# 다운받은 ssh key에 권한 설정
chmod 400 ~/Downloads/ssh-key-XXXX.key

# VM 접속 (유저명은 ubuntu)
ssh -i ~/Downloads/ssh-key-XXXX.key ubuntu@<PUBLIC_IP>
```

### 2.1 방화벽 (iptables) 열기

Oracle Ubuntu 이미지는 OS 레벨 iptables가 80/443을 막고 있음:

```bash
sudo iptables -I INPUT 6 -m state --state NEW -p tcp --dport 80 -j ACCEPT
sudo iptables -I INPUT 6 -m state --state NEW -p tcp --dport 443 -j ACCEPT
sudo netfilter-persistent save
```

### 2.2 Docker 설치

```bash
sudo apt update
sudo apt install -y docker.io docker-compose-v2 git
sudo usermod -aG docker $USER
newgrp docker  # 재로그인 없이 그룹 적용
```

---

## 3. 도메인 받기 (DuckDNS 무료)

1. https://www.duckdns.org 에 GitHub로 로그인
2. 원하는 서브도메인 입력 (예: `commitcat`) → **add domain**
3. `commitcat.duckdns.org`에 VM의 **public IP**를 입력 → **update ip**
4. 페이지 상단의 **token** 값 복사해둠 (자동 갱신용, 선택사항)

---

## 4. 프로젝트 clone & 설정

VM에서:

```bash
git clone https://github.com/eunseo9311/commit-cat.git
cd commit-cat/deploy

cp .env.example .env
nano .env
```

`.env`에 아래 값 채우기:

```
JWT_SECRET=아무_긴_랜덤_문자열_50자이상
GITHUB_CLIENT_ID=<GitHub OAuth App Client ID>
GITHUB_CLIENT_SECRET=<GitHub OAuth App Client Secret>
REDIRECT_URI=https://commitcat.duckdns.org/auth/github/callback
DOMAIN=commitcat.duckdns.org
EMAIL=your-email@example.com
```

### 4.1 GitHub OAuth App 만들기

https://github.com/settings/developers → **New OAuth App**

| 항목 | 값 |
|---|---|
| Application name | CommitCat |
| Homepage URL | `https://commitcat.duckdns.org` |
| Authorization callback URL | `https://commitcat.duckdns.org/auth/github/callback` |

Client ID / Client Secret을 `.env`에 붙여넣기.

---

## 5. 초기 배포 (Let's Encrypt 인증서 포함)

```bash
chmod +x init-letsencrypt.sh deploy.sh
./init-letsencrypt.sh
```

스크립트가:
1. 임시 self-signed 인증서 생성
2. nginx 기동
3. Let's Encrypt 실제 인증서 발급 (HTTP challenge)
4. nginx reload
5. 전체 스택 기동

성공하면 https://commitcat.duckdns.org 접속 가능.

### 확인

```bash
curl https://commitcat.duckdns.org/health
# → ok

curl -I https://commitcat.duckdns.org/badge/eunseo9311
# → 200 OK, content-type: image/svg+xml
```

---

## 6. 코드 업데이트 시 재배포

```bash
cd ~/commit-cat/deploy
./deploy.sh
```

`git pull` → 서버 이미지만 재빌드 → nginx는 건드리지 않고 서버만 재기동.

---

## 7. README 뱃지 URL 교체

로컬에서:

```bash
sed -i '' 's|commitcat-api.fly.dev|commitcat.duckdns.org|g' README.md
git commit -am "chore: 뱃지 URL 오라클 클라우드로 변경"
git push
```

---

## 8. 인증서 자동 갱신

`docker-compose.yml`의 `certbot` 컨테이너가 12시간마다 `certbot renew`를 실행해서 자동 갱신됨. 별도 작업 불필요.

---

## 문제 해결

### 502 Bad Gateway
```bash
docker compose logs commitcat-api   # 서버 로그
docker compose logs nginx           # nginx 로그
docker compose ps                   # 컨테이너 상태
```

### 인증서 발급 실패
- DuckDNS에서 IP가 올바른지 확인: `dig commitcat.duckdns.org`
- VM의 iptables 80/443 열려있는지 확인
- Oracle Security List 열려있는지 확인

### 디스크 풀
```bash
docker system prune -a  # 안 쓰는 이미지 정리
```
