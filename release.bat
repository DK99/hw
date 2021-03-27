echo "Exporting token and enterprise api to enable github-release tool"

echo "Deleting release from github before creating new one"
F:\github-release.exe delete --user allesctf --repo hw --tag allesctf_edition_windows

echo "Creating a new release in github"
F:\github-release.exe release --user allesctf --repo hw --tag allesctf_edition_windows --name allesctf_edition_windows

echo "Uploading the artifacts into github"
F:\github-release.exe upload --user allesctf --repo hw --tag allesctf_edition_windows --name "hedgewars_allesctf_edition_windows.zip" --file hedgewars.zip