<template>
  <h1>Items</h1>
  <div v-if="data">
    <table>
        <tr>
          <th>ID</th>
          <th>SKU</th>
          <th>Name</th>
          <th>Quantity</th>
        </tr>
        <tr v-for="item in data.items" :key="item.id">
          <td>{{ item.id }}</td>
          <td>{{ item.sku ?? '--' }}</td>
          <td>{{ item.name }}</td>
          <td>{{ item.quantity }}</td>
        </tr>
    </table>
  </div>
</template>
<script>
import { useQuery, useSubscription } from "@urql/vue";

export default {
  setup() {
    const handleSubscription = (items, response) => {
      console.log("SUBSCRIPTION");
      console.log(items);
      console.log(response);
      const result = response.itemSubscription;
      if (result.modification === "CREATE") {
        return [result.item, ...items];
      } else if (result.modification === "UPDATE") {
        const foundIndex = items.findIndex(item => item.id === result.item.id);
        items[foundIndex] = result.item;
        return items;
      } else if (result.modification === "DELETE") {
        return items.filter(item => item.id !== result.item.id);
      }
    }

    const result = useQuery({
      query: `
      {
        items {
          id,
          sku,
          name,
          quantity,
        }
      }
      `
    });

    const subscription = useSubscription({
      query: `
      subscription {
        itemSubscription {
          id,
          sku,
          name,
          quantity
        }
      }
      `
    }, handleSubscription);

    return {
      data: result.data,
      updates: subscription.data,
    };
  }
};
</script>