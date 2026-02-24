targets=(
    ".:README.md"
    "apps/desktop:apps/desktop/README.md"
    "apps/frontend:apps/frontend/README.md"
    "apps/backend:apps/backend/README.md"
)

for target in "${targets[@]}"; do
    dir="${target%%:*}"
    file="${target#*:}"
    if [ -f "$file" ]; then
        echo "Updating $file..."
        echo '[TREE-START]' > temp_tree.txt
        echo '```text' >> temp_tree.txt
        tree "$dir" -I 'node_modules|dist|target|.git|.github|.vscode|build|.DS_Store|temp_tree.txt' -L 3 --dirsfirst --noreport >> temp_tree.txt
        echo '```' >> temp_tree.txt
        echo '[TREE-END]' >> temp_tree.txt
        perl -0777 -i -pe 's/\[TREE-START\].*?\[TREE-END\]/`cat temp_tree.txt`/ges' "$file"
        rm temp_tree.txt
    fi
done