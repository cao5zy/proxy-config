
#!/bin/bash

# upload.sh - 同步上传文件到远程服务器
# 用法: ./upload.sh <地址> <用户名>

# 检查参数数量
if [ $# -ne 2 ]; then
    echo "错误: 需要两个参数 - 地址和用户名"
    echo "用法: $0 <地址> <用户名>"
    exit 1
fi

REMOTE_ADDRESS="$1"
USERNAME="$2"
REMOTE_DIR="~/proxy-config"

# 检查当前目录是否存在
if [ ! -d "$(pwd)" ]; then
    echo "错误: 当前目录不存在"
    exit 1
fi

echo "开始同步上传文件到 $USERNAME@$REMOTE_ADDRESS:$REMOTE_DIR"

# 使用rsync进行同步上传
# -a: 归档模式，保持文件属性
# -v: 详细输出
# -z: 压缩传输
# --delete: 删除目标目录中多余的文件
# --exclude: 排除不需要上传的文件
rsync -avz --delete \
    --exclude=".git/" \
    --exclude="target/" \
    --exclude="*.log" \
    --exclude="node_modules/" \
    --exclude="Cargo.lock" \
    --exclude="proxy-config.yml" \
    . "$USERNAME@$REMOTE_ADDRESS:$REMOTE_DIR"

# 检查rsync执行结果
if [ $? -eq 0 ]; then
    echo "文件同步上传成功！"
    echo "ssh $USERNAME@$REMOTE_ADDRESS"
    exit 0
else
    echo "错误: 文件同步上传失败"
    exit 1
fi
