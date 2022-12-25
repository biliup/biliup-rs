# docker build . --file Dockerfile --tag $IMAGE_NAME
# echo "${{ secrets.GITHUB_TOKEN }}" | docker login ghcr.io -u ${{ github.actor }} --password-stdin
# docker tag $IMAGE_NAME $IMAGE_ID:$VERSION
# docker push $IMAGE_ID:$VERSION
FROM messense/manylinux2014-cross:x86_64

RUN apt-get install -y libssl-dev

