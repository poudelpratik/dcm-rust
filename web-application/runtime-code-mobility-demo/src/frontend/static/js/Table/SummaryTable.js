'use strict';

import { convertHtmlStringToElement, euro } from '../utils/helpers.js';
import Table from './index.js';
import { createTableFooterHTMLTemplate } from '../utils/tableFooterTemplate.js';
import {add_floats, calc_discount, round_float} from '../initCodeDistributor.js';

export default class SummaryTable extends Table {
  /**
   * Creates an instance of CartTable.
   * @param {Object} tableOptions
   * @param {Object} total
   * @param {Number} total.value
   * @param {String} total.price
   * @param {Boolean} isCouponActive
   * @memberof CartTable
   */
  constructor(tableOptions, total, isCouponActive) {
    tableOptions = tableOptions ?? {};
    super(tableOptions);
    this.total = total;
    this.isCouponActive = isCouponActive;

    this.scrollHeight = document.body.scrollHeight;
  }

  /**
   * Initial load of the table.
   */
  renderCosts = async () => {
    this.removeAllTableRows();
    /** @type {string[]} */
    let rows = [];
    /** @type {Number[]} */
    let totalValues = [];

    totalValues.push(this.total.value);
    rows.push(
      this.#createTableRow({
        name: 'Total price',
        priceObj: this.total,
      })
    );

    if (this.isCouponActive) {
      // toFixed returns a string => convert with +
      const discountValue = await calc_discount(
        this.total.value,
        15.0
      );
      const discountValuePrecisionTwo = await round_float(
        discountValue
      );

      totalValues.push(-discountValuePrecisionTwo);

      rows.push(
        this.#createTableRow({
          name: 'Voucher (15% discount)',
          priceObj: {
            value: -discountValuePrecisionTwo,
            price: '-' + euro(discountValuePrecisionTwo).format(),
          },
        })
      );
    }

    const transportCostValue = 6.0;
    totalValues.push(transportCostValue);

    rows.push(
      this.#createTableRow({
        name: 'Transport Cost',
        priceObj: {
          value: transportCostValue,
          price: euro(transportCostValue).format(),
        },
      })
    );

    const total = await add_floats(totalValues);

    this.tableBody.append(...rows);

    let footerRow = convertHtmlStringToElement(
      createTableFooterHTMLTemplate(
        {
          value: total,
          price: euro(total).format(),
        },
        2
      )
    );

    this.tableFooter.append(footerRow);

    window.scrollTo(0, this.scrollHeight);
  };

  #createTableRow = (data) => {
    const { name, priceObj } = data;
    return convertHtmlStringToElement(`
      <tr>
        <td data-label="Kostenpunkt">${name}</td>
        <td data-label="Preis" data-value="${priceObj.value}">${priceObj.price}</td>
      </tr>
    `);
  };
}
