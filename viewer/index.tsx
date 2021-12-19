import { createSignal, createEffect, Component } from "solid-js";
import { render } from "solid-js/web";
import nacl from 'tweetnacl';
import Handlebars from 'handlebars';

const App: Component = () => {
  // web4_get in the contract will replace these with JSON values
  const certificateId = "{{{ CERTIFICATE_ID }}};
  const certificate = {{{ CERTIFICATE }}};
  const certificateTemplate = {{{ CERTIFICATE_TEMPLATE }}};

  // TODO: handle missing key
  const certificateKey = Buffer.from(window.location.hash.slice(1), 'hex');

  /*
  const [renderedCertificate, setRenderedCertificate] = createSignal('');
  createEffect(() => {
    fetch(
      "https://rpc.testnet.near.org", 
      {
        headers: {
          'Accept': 'application/json',
          'Content-Type': 'application/json'
        },
        method: "POST",
        body: JSON.stringify({
          jsonrpc: "2.0",
          id: "dontcare",
          method: "query",
          params: {
            request_type: "call_function",
            account_id: "cert.frol4.testnet",
            method_name: "get_certificate",
            args_base64: btoa(JSON.stringify({
              certificate_id: certificateId
            })),
            finality: "final",
          }
        })
      }
    )
    .then((data) => data.json())
    .then((jsonData) => {
      const certificate = JSON.parse(Buffer.from(jsonData.result.result));
      const encryptedCertificateData = Buffer.from(certificate.encrypted_certificate_data);
      const certificateEncryptionNonce = Buffer.from(certificate.certificate_encryption_nonce);
      const decryptedCertificateData = nacl.secretbox.open(encryptedCertificateData, certificateEncryptionNonce, certificateKey);
      const certificateData = JSON.parse(Buffer.from(decryptedCertificateData));
      const certificateTemplate = Handlebars.compile(certificateData.certificate_template);
      setRenderedCertificate(certificateTemplate(certificateData))
    })
  })*/

  const encryptedCertificateData = Buffer.from(certificate.encrypted_certificate_data);
  const certificateEncryptionNonce = Buffer.from(certificate.certificate_encryption_nonce);
  const decryptedCertificateData = nacl.secretbox.open(encryptedCertificateData, certificateEncryptionNonce, certificateKey);
  const certificateData = JSON.parse(Buffer.from(decryptedCertificateData));
  const certificateTemplate = Handlebars.compile(certificateData.certificate_template);
  const renderedCertificate = certificateTemplate(certificateData);

  return <div innerHTML={renderedCertificate}></div>;
};

render(() => <App />, document.getElementById("app"));
