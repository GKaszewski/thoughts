FROM python:3.12
WORKDIR /usr/src/thoughts

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .

RUN apt-get update && apt-get install -y cron
RUN echo "0 0 * * * cd /usr/src/thoughts && python3 manage.py delete_expired_thoughts >> /var/log/cron.log 2>&1" > /etc/crontab

CMD ["cron", "-f"]