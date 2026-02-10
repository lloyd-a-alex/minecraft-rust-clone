$date = Get-Date -Format "dd/MM/yyyy HH:mm:ss"
git add .
git commit -m "AI Update: $date"
git push origin main
cargo run