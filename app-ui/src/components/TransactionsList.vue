<template>
  <div v-if="live?.length > 0 || base?.length > 0">
    <table>
      <tr>
        <th>ID</th>
        <th>Item ID</th>
        <th>Item SKU</th>
        <th>Item Name</th>
        <th>Location ID</th>
        <th>Location Name</th>
        <th>Location Address</th>
        <th>Transaction Date</th>
        <th>Quantity</th>
        <th>Comment</th>
      </tr>
      <tr v-for="transaction in live ?? base" :key="transaction.id">
        <td>{{ transaction.id }}</td>
        <td>{{ transaction.item.id }}</td>
        <td>{{ transaction.item.sku ?? '--' }}</td>
        <td>{{ transaction.item.name }}</td>
        <td>{{ transaction.location.id ?? '--' }}</td>
        <td>{{ transaction.location.name ?? '--' }}</td>
        <td>{{ transaction.location.address ?? '--' }}</td>
        <td>{{ transaction.transactionDate.split('T')[0] }}</td>
        <td>{{ transaction.quantity }}</td>
        <td>{{ transaction.comment ?? '--' }}</td>
      </tr>
    </table>
  </div>
  <div v-else>
    <p>No transactions added to the inventory tracking system.</p>
  </div>
</template>
<script>
import { useClientHandle } from "@urql/vue";

export default {
  async setup() {
    const handle = useClientHandle();

    // get the initial transactions
    let result = await handle
      .useQuery({
        query: `
      {
        transactions {
          id,
          item {
            id,
            sku,
            name,
          },
          location {
            id,
            name,
            address,
          },
          transactionDate,
          quantity,
          comment,
        }
      }
      `,
      })
      .executeQuery();
    result = JSON.parse(JSON.stringify(result.data.value.transactions));

    // update the transactions live
    const handleSubscription = (transactions = result, response) => {
      const event = response.transactionSubscription;
      if (event.modification === "CREATE") {
        return [...transactions, event.data];
      } else if (event.modification === "UPDATE") {
        const foundIndex = transactions.findIndex((transaction) => transaction.id === event.data.id);
        transactions[foundIndex] = event.data;
        return transactions;
      } else if (event.modification === "DELETE") {
        return transactions.filter((transaction) => transaction.id !== event.data.id);
      }
    };

    // subscribe to transaction changes
    const subscription = handle.useSubscription(
      {
        query: `
      subscription {
        transactionSubscription {
          modification,
          data {
            id,
            item {
              id,
              sku,
              name,
            },
            location {
              id,
              name,
              address,
            },
            transactionDate,
            quantity,
            comment,
          }
        }
      }
      `,
      },
      handleSubscription
    );

    // hack to get things working
    return {
      base: result,
      live: subscription.data,
    };
  },
};
</script>