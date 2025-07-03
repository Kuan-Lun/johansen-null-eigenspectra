#!/bin/bash

# DAT 檔案編碼品質分析腳本
# 此腳本不修改任何專案檔案，僅進行分析
# 
# 依賴工具：bc, pigz, pbzip2, xz (用於並行壓縮和浮點數計算)
# 如果未安裝，請先執行：apt install bc pigz pbzip2 xz-utils

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
overall_highest_ratio=0  # 追蹤最高壓縮率(代表原始編碼效率最低的檔案)
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
    echo "方法          壓縮後大小     壓縮率"
    echo "------------------------------------"
    
    # 獲取 CPU 核心數
    cpu_cores=$(nproc)
    
    # gzip 壓縮測試 (使用 pigz 並行版本)
    pigz -c -1 -p $cpu_cores "$datfile" > "${datfile}.gz1.tmp"
    gz1_size=$(stat -c%s "${datfile}.gz1.tmp")
    gz1_ratio=$(echo "scale=1; (1 - $gz1_size / $original_size) * 100" | bc)
    printf "gzip -1       %-13s  %5s%%\n" "$(numfmt --to=iec $gz1_size)B" "$gz1_ratio"
    
    pigz -c -9 -p $cpu_cores "$datfile" > "${datfile}.gz9.tmp"
    gz9_size=$(stat -c%s "${datfile}.gz9.tmp")
    gz9_ratio=$(echo "scale=1; (1 - $gz9_size / $original_size) * 100" | bc)
    printf "gzip -9       %-13s  %5s%%\n" "$(numfmt --to=iec $gz9_size)B" "$gz9_ratio"
    
    # bzip2 壓縮測試 (使用 pbzip2 並行版本)
    pbzip2 -c -9 -p$cpu_cores "$datfile" > "${datfile}.bz2.tmp"
    bz2_size=$(stat -c%s "${datfile}.bz2.tmp")
    bz2_ratio=$(echo "scale=1; (1 - $bz2_size / $original_size) * 100" | bc)
    printf "bzip2 -9      %-13s  %5s%%\n" "$(numfmt --to=iec $bz2_size)B" "$bz2_ratio"
    
    # xz 壓縮測試 (使用多線程)
    xz -c -6 -T 0 "$datfile" > "${datfile}.xz.tmp"
    xz_size=$(stat -c%s "${datfile}.xz.tmp")
    xz_ratio=$(echo "scale=1; (1 - $xz_size / $original_size) * 100" | bc)
    printf "xz -6         %-13s  %5s%%\n" "$(numfmt --to=iec $xz_size)B" "$xz_ratio"
    
    # 清理臨時檔案
    rm -f "${datfile}".*.tmp
    
    echo
    echo "編碼品質評估："
    
    # 根據最佳壓縮率評估
    best_ratio=$(echo "$gz1_ratio $gz9_ratio $bz2_ratio $xz_ratio" | tr ' ' '\n' | sort -nr | head -1)
    
    # 追蹤最高壓縮率(用於找出編碼效率最低的檔案)
    if [ $(echo "$best_ratio > $overall_highest_ratio" | bc) -eq 1 ]; then
        overall_highest_ratio=$best_ratio
    fi
    total_files=$((total_files + 1))
    
    if [ $(echo "$best_ratio < 10" | bc) -eq 1 ]; then
        echo "[優秀] 原始編碼極高效，壓縮率低於 10%，幾乎達到 f64 數據理論極限"
    elif [ $(echo "$best_ratio < 20" | bc) -eq 1 ]; then
        echo "[良好] 原始編碼效率很好，壓縮率 10-20%，接近 f64 數據理論極限"
    elif [ $(echo "$best_ratio < 35" | bc) -eq 1 ]; then
        echo "[中等] 原始編碼尚可，壓縮率 20-35%，屬於典型科學數據表現"
    elif [ $(echo "$best_ratio < 50" | bc) -eq 1 ]; then
        echo "[普通] 原始編碼有改進空間，壓縮率 35-50%，可優化數據組織結構"
    else
        echo "[需改進] 原始編碼效率低，壓縮率高於 50%，存在明顯冗餘或格式問題"
    fi
    
    echo
done

echo "=========================================="
echo "=== 總結與建議 ==="
echo "=========================================="
echo
echo "測試結果摘要："
echo "- 分析了 $total_files 個 dat 檔案"
echo "- 最高壓縮率: ${overall_highest_ratio}% (代表原始編碼效率最低的檔案)"
echo

# 根據最高壓縮率評估 (保守策略，關注編碼效率最低的檔案)
if [ $(echo "$overall_highest_ratio < 10" | bc) -eq 1 ]; then
    echo "[整體評估: 優秀]"
    echo "你的 dat 檔案編碼已達到極高效率，幾乎達到 f64 數據的理論壓縮極限。"
    echo "建議: 保持現有格式，專注於讀寫效能優化和並行處理。"
elif [ $(echo "$overall_highest_ratio < 20" | bc) -eq 1 ]; then
    echo "[整體評估: 良好]"
    echo "你的 dat 檔案編碼效率很好，已接近 f64 數據的理論壓縮極限。"
    echo "建議: 當前編碼已很優秀，可考慮在此基礎上優化存取模式和緩存策略。"
elif [ $(echo "$overall_highest_ratio < 35" | bc) -eq 1 ]; then
    echo "[整體評估: 中等]"
    echo "你的 dat 檔案編碼效率屬於典型科學數據表現，在合理範圍內。"
    echo "注意: 較小的檔案通常壓縮效率較低，這是正常現象。"
    echo "建議: 可考慮區塊壓縮、差分編碼或重新排列數據以提高局部性。"
elif [ $(echo "$overall_highest_ratio < 50" | bc) -eq 1 ]; then
    echo "[整體評估: 普通]"
    echo "你的 dat 檔案編碼有明顯改進空間，可通過優化獲得更好效率。"
    echo "建議: 評估數據分佈特性，考慮特化的浮點數編碼方案或內建壓縮。"
else
    echo "[整體評估: 需要改進]"
    echo "你的 dat 檔案編碼效率較低，存在明顯冗餘或格式設計問題。"
    echo "建議: 檢查檔案格式設計，考慮專用的科學數據格式如 HDF5 或 NetCDF。"
fi

echo
echo "編碼品質參考標準 (針對完整 f64 精度二進制浮點數資料)："
echo "說明: f64 浮點數理論上難以高效壓縮，典型科學數據壓縮率在 15-25% 範圍"
echo "注意: 壓縮率越低代表原始編碼越高效，因為壓縮工具找不到太多可壓縮的空間"
echo
echo "- 壓縮率 < 10%: 優秀 (原始編碼極高效，幾乎無法進一步壓縮)"
echo "- 壓縮率 10-20%: 良好 (編碼效率很好，接近 f64 數據理論極限)"
echo "- 壓縮率 20-35%: 中等 (編碼效率尚可，典型科學數據表現)"
echo "- 壓縮率 35-50%: 普通 (有明顯改進空間，可優化數據組織)"
echo "- 壓縮率 > 50%: 需改進 (編碼效率低，存在大量冗餘或格式問題)"