#!/bin/bash

# DAT 檔案編碼品質分析腳本
# 此腳本不修改任何專案檔案，僅進行分析

echo "=== DAT 檔案編碼品質分析 ==="
echo

cd /usr/src/app/data

# 檢查是否有 dat 檔案
if ! ls *.dat >/dev/null 2>&1; then
    echo "錯誤：找不到 .dat 檔案"
    exit 1
fi

echo "現有 dat 檔案："
ls -lh *.dat
echo

# 對每個 dat 檔案進行分析
overall_best_ratio=100
overall_best_saved=0
total_files=0
for datfile in *.dat; do
    echo "=========================================="
    echo "分析檔案: $datfile"
    echo "=========================================="
    
    # 獲取檔案大小
    original_size=$(stat -c%s "$datfile")
    echo "原始大小: $(numfmt --to=iec $original_size)B ($original_size bytes)"
    
    # 檔案格式分析
    echo
    echo "檔案格式分析："
    echo "- 前 16 bytes (hex):"
    head -c 16 "$datfile" | od -t x1 -A x
    
    echo "- Magic header (前 12 bytes):"
    head -c 12 "$datfile" | od -t c -A n | tr -d ' \n'
    echo
    
    # 壓縮率測試
    echo
    echo "壓縮率測試："
    echo "方法          壓縮後大小     壓縮率    空間節省"
    echo "--------------------------------------------"
    
    # gzip 壓縮測試
    gzip -c -1 "$datfile" > "${datfile}.gz1.tmp"
    gz1_size=$(stat -c%s "${datfile}.gz1.tmp")
    gz1_ratio=$(python3 -c "print(f'{$gz1_size * 100 / $original_size:.1f}')")
    gz1_saved=$(python3 -c "print(f'{100 - $gz1_size * 100 / $original_size:.1f}')")
    printf "gzip -1       %-13s  %5s%%    %5s%%\n" "$(numfmt --to=iec $gz1_size)B" "$gz1_ratio" "$gz1_saved"
    
    gzip -c -9 "$datfile" > "${datfile}.gz9.tmp"
    gz9_size=$(stat -c%s "${datfile}.gz9.tmp")
    gz9_ratio=$(python3 -c "print(f'{$gz9_size * 100 / $original_size:.1f}')")
    gz9_saved=$(python3 -c "print(f'{100 - $gz9_size * 100 / $original_size:.1f}')")
    printf "gzip -9       %-13s  %5s%%    %5s%%\n" "$(numfmt --to=iec $gz9_size)B" "$gz9_ratio" "$gz9_saved"
    
    # bzip2 壓縮測試
    bzip2 -c -9 "$datfile" > "${datfile}.bz2.tmp"
    bz2_size=$(stat -c%s "${datfile}.bz2.tmp")
    bz2_ratio=$(python3 -c "print(f'{$bz2_size * 100 / $original_size:.1f}')")
    bz2_saved=$(python3 -c "print(f'{100 - $bz2_size * 100 / $original_size:.1f}')")
    printf "bzip2 -9      %-13s  %5s%%    %5s%%\n" "$(numfmt --to=iec $bz2_size)B" "$bz2_ratio" "$bz2_saved"
    
    # xz 壓縮測試
    xz -c -6 "$datfile" > "${datfile}.xz.tmp"
    xz_size=$(stat -c%s "${datfile}.xz.tmp")
    xz_ratio=$(python3 -c "print(f'{$xz_size * 100 / $original_size:.1f}')")
    xz_saved=$(python3 -c "print(f'{100 - $xz_size * 100 / $original_size:.1f}')")
    printf "xz -6         %-13s  %5s%%    %5s%%\n" "$(numfmt --to=iec $xz_size)B" "$xz_ratio" "$xz_saved"
    
    # 清理臨時檔案
    rm -f "${datfile}".*.tmp
    
    echo
    echo "編碼品質評估："
    
    # 根據最佳壓縮率評估
    best_ratio=$(python3 -c "
ratios = [$gz1_ratio, $gz9_ratio, $bz2_ratio, $xz_ratio]
print(min(ratios))
")
    best_saved=$(python3 -c "print(f'{100 - $best_ratio:.1f}')")
    
    # 更新全局最佳結果
    if python3 -c "exit(0 if $best_ratio < $overall_best_ratio else 1)"; then
        overall_best_ratio=$best_ratio
        overall_best_saved=$best_saved
    fi
    total_files=$((total_files + 1))
    
    if python3 -c "exit(0 if $best_ratio < 30 else 1)"; then
        echo "✓ 優秀：編碼非常高效，壓縮率低於 30% (節省超過 70% 空間)"
    elif python3 -c "exit(0 if $best_ratio < 50 else 1)"; then
        echo "✓ 良好：編碼效率不錯，壓縮率 30-50% (節省 50-70% 空間)"
    elif python3 -c "exit(0 if $best_ratio < 70 else 1)"; then
        echo "✓ 中等：編碼效率尚可，壓縮率 50-70% (節省 30-50% 空間)"
    elif python3 -c "exit(0 if $best_ratio < 85 else 1)"; then
        echo "⚠ 普通：編碼有改進空間，壓縮率 70-85% (節省 15-30% 空間)"
    else
        echo "⚠ 需改進：編碼效率較低，壓縮率超過 85% (節省空間不足 15%)"
    fi
    
    echo
done

echo "=========================================="
echo "=== 總結與建議 ==="
echo "=========================================="
echo
echo "📊 測試結果摘要："
echo "- 分析了 $total_files 個 dat 檔案"
echo "- 最佳壓縮率: ${overall_best_ratio}% (節省 ${overall_best_saved}% 空間)"
echo

# 根據實際結果給出具體評估
if python3 -c "exit(0 if $overall_best_ratio < 30 else 1)"; then
    echo "🎉 整體評估: 優秀"
    echo "你的 dat 檔案編碼非常高效，已經接近最優狀態。"
    echo "建議: 保持現有格式，專注於讀寫效能優化。"
elif python3 -c "exit(0 if $overall_best_ratio < 50 else 1)"; then
    echo "✅ 整體評估: 良好"
    echo "你的 dat 檔案編碼效率不錯，只有小幅改進空間。"
    echo "建議: 可考慮在需要時添加可選壓縮功能。"
elif python3 -c "exit(0 if $overall_best_ratio < 70 else 1)"; then
    echo "👍 整體評估: 中等"
    echo "你的 dat 檔案編碼效率尚可，有一定改進空間。"
    echo "建議: 可考慮差分編碼或內建壓縮來進一步優化。"
elif python3 -c "exit(0 if $overall_best_ratio < 85 else 1)"; then
    echo "⚠️ 整體評估: 普通"
    echo "你的 dat 檔案編碼有改進空間，但對於 f64 浮點數來說仍在合理範圍內。"
    echo "建議: 評估是否需要完整的 f64 精度，或考慮添加壓縮層。"
else
    echo "❌ 整體評估: 需要改進"
    echo "你的 dat 檔案編碼效率較低，建議重新設計格式。"
    echo "建議: 考慮專用的 eigenvalue 編碼格式或強制壓縮。"
fi

echo
echo "📋 編碼品質參考標準 (針對二進制浮點數資料)："
echo "- 壓縮率 < 30%: 優秀 (節省 > 70% 空間)"
echo "- 壓縮率 30-50%: 良好 (節省 50-70% 空間)"
echo "- 壓縮率 50-70%: 中等 (節省 30-50% 空間)"
echo "- 壓縮率 70-85%: 普通 (節省 15-30% 空間)"
echo "- 壓縮率 > 85%: 需改進 (節省空間 < 15%)"
