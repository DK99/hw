
#!/bin/bash

echo "Exporting token and enterprise api to enable github-release tool"

echo "Deleting release from github before creating new one"
/home/jenkins/go/bin/github-release delete --user allesctf --repo hw --tag allesctf_edition_linux

git tag allesctf_edition_linux
git push --tags "https://${GITHUB_TOKEN}@github.com/allesctf/hw"

echo "Creating a new release in github"
/home/jenkins/go/bin/github-release release --user allesctf --repo hw --tag allesctf_edition_linux --name allesctf_edition_linux

echo "Uploading the artifacts into github"
/home/jenkins/go/bin/github-release upload --user allesctf --repo hw --tag allesctf_edition_linux --name "hedgewars_allesctf_edition_linux.zip" --file hedgewars.zip