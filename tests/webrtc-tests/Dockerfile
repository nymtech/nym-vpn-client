FROM node:18-slim

RUN apt-get update && apt-get install -y \
    libnss3 \
    libatk-bridge2.0-0 \
    libdrm2 \
    libxkbcommon0 \
    libxcomposite1 \
    libxdamage1 \
    libxrandr2 \
    libgbm1 \
    libxss1 \
    libasound2 \
    libatspi2.0-0 \
    libgtk-3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app/webrtc-tests

COPY package*.json ./

RUN npm install

RUN npx playwright install chromium

COPY webrtc-leak-test.js ./

RUN mkdir -p ../results/webrtc

RUN chmod +x webrtc-leak-test.js

CMD ["node", "webrtc-leak-test.js"] 