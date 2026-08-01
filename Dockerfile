FROM python:3.11-slim
WORKDIR /app
ENV PYTHONUNBUFFERED=1
ENV PORT=8080
COPY . /app
EXPOSE 8080
CMD ["python3", "scripts/pos_backend.py"]
