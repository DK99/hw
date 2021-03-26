
#!/bin/bash

echo "Exporting token and enterprise api to enable github-release tool"

echo "Deleting release from github before creating new one"
github-release delete --user allesctf --repo hw --tag allesctf_edition

echo "Creating a new release in github"
github-release release --user allesctf --repo hw --tag allesctf_edition --name allesctf_edition

echo "Uploading the artifacts into github"
github-release upload --user allesctf --repo hw --tag allesctf_edition --name "hedgewars_allesctf_edition_linux.zip" --file hedgewars.zip