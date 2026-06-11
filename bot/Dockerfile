FROM node:22-slim
RUN groupadd -r daglock && useradd -r -g daglock -m -d /home/daglock daglock
WORKDIR /app
COPY package*.json ./
RUN npm ci --production
COPY src/ src/
RUN chown -R daglock:daglock /app
USER daglock
CMD ["node", "src/index.js"]
