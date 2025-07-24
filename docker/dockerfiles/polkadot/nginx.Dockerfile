FROM nginx:alpine

# Copy custom nginx config file (if you have one)
# COPY nginx.conf /etc/nginx/nginx.conf

# Copy static content (if any)
# COPY ./html /usr/share/nginx/html

# Expose port 80
EXPOSE 80

# Start Nginx
CMD ["nginx", "-g", "daemon off;"]
