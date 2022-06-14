const tokenDecimals = {
  "eICP": 12,
  "eXTC": 12,
};

export const bigIntToFloat = (value, token, decimals) => {
  if (typeof decimals === 'undefined') {
    decimals = 6;
  }
  let tokenDecimal = tokenDecimals[token];
  if (!tokenDecimal) return NaN;
  let val = value * BigInt("1" + "0".repeat(decimals)) / BigInt("1" + "0".repeat(tokenDecimal));
  let str = val.toString();
  return parseFloat(str.slice(0, -decimals) + "." + str.slice(-decimals));
}

export const bigIntToStr = (value, token, decimals, defaultVal) => {
  if (typeof defaultVal === 'undefined') {
    defaultVal = null;
  }
  let ret = bigIntToFloat(value, token, decimals);
  return isNaN(ret) ? defaultVal : ret.toString();
}

// use significant digits due to javascript floating point error
export const floatToBigInt = (value, token, significantDigits = 8) => {
  let tokenDecimal = tokenDecimals[token];
  if (!tokenDecimal) return null;
  if (tokenDecimal <= significantDigits) {
    return BigInt(value.toFixed(tokenDecimal).replace(/\./g, ''));
  } else {
    return BigInt(value.toFixed(significantDigits).replace(/\./g, '') + '0'.repeat(tokenDecimal - significantDigits));
  }
}
