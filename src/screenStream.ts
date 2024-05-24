export class ScreenStream {
  img: HTMLImageElement;
  clientId: string;
  connectTimeoutId: number | undefined;

  public constructor(imgElement: HTMLImageElement, clientId: string) {
    this.img = imgElement;
    this.clientId = clientId;
  }

  public connect(address: string, onConnect: () => void, onError: () => void) {
    if (address.endsWith("/")) address = address.slice(0, -1);
    const that = this;
    const img = that.img;
    const url = `${address}/stream.mjpeg?clientId=${this.clientId}`;

    img.src = "";
    clearTimeout(that.connectTimeoutId);
    new Promise<void>((resolve, reject) => {
      img.onload = function () {
        img.onload = null;
        img.onerror = null;
        resolve();
      };
      img.onerror = function (e) {
        img.onerror = null;
        img.onload = null;
        reject(e);
      };
      img.src = url;
    })
      .then(() => {
        onConnect();
      })
      .catch(() => {
        img.src = "";
        onError();
      });
  }
}
